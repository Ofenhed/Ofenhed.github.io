use leptos::{attr::custom::custom_attribute, html, prelude::*};
use leptos_router::{LazyRoute, lazy_route};

use crate::helpers::{ImgDef, NoScript, NoWasm};

#[cfg(not(feature = "ssr"))]
mod qr_settings {
    pub const MIN_INITIAL_DELAY: f64 = 0.250;
    pub const MAX_INITIAL_DELAY: f64 = 1.0;
    pub const SHADE_STEP_SIZE: u8 = 32;
    pub const MIN_SHADE_INTERVAL: f64 = 0.05;
    pub const MAX_SHADE_INTERVAL: f64 = 0.1;
    pub const MIN_WORM_LENGTH: usize = 20;
    pub const MAX_WORM_LENGTH: usize = 250;
    pub const MIN_WORM_INTERVAL: f64 = 2.0;
    pub const MAX_WORM_INTERVAL: f64 = 10.0;
    pub const WORM_SHADE_STEPS: usize = 5;
    pub const WORM_SHADE_INTERVAL: f64 = 0.08;
    pub const WORM_MOVE_INTERVAL: f64 = 0.02;
    pub const WORM_MOVE_ANGLE: f64 = 0.75;
    const _COMPILE_TIME_ASSERTIONS: () = {
        assert!(MAX_INITIAL_DELAY > MIN_INITIAL_DELAY);
        assert!(MAX_SHADE_INTERVAL > MIN_SHADE_INTERVAL);
        assert!(MAX_WORM_INTERVAL > MIN_WORM_INTERVAL);
        assert!(MAX_WORM_LENGTH > MIN_WORM_LENGTH);
    };
}

macro_rules! make_vcard {
    ( $vcard:literal, [$($logo_line:literal),*] ) => {
        #[allow(unused)]
        const VCARD: &str = $vcard;
        cfg_select! {
            debug_assertions => qr_macro::make_qr! {
                const QR_CODE = env!($vcard) || "NO VCARD" + [ $($logo_line),* ];
            },
            _ => qr_macro::make_qr! {
                const QR_CODE = env!($vcard) + [ $($logo_line),* ];
            }
        }
    };
}

make_vcard!(
    "contact_card",
    [
        "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
        "xooooooxooooooxooooooxoooooxxooxooooooxooxooooooxoooooox",
        "xooxxxxxooxxooxooxxooxooxxooxooxxxooxxxooxooxxooxooxxoox",
        "xooxxxxxooxxooxooxxooxooxxooxooxxxooxxxooxooxxooxooxxoox",
        "xooxxxxxooxxooxooxxooxooxxooxooxxxooxxxooxooxxooxooxxoox",
        "xooooooxooooooxooxxooxoooooxxooxxxooxxxooxooooooxooxxoox",
        "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
        "////////////xooooooxooooooxooxooooooxoooooox////////////",
        "////////////xooxxooxxxxxooxooxooxxxxxooxxoox////////////",
        "////////////xoooooxxooooooxooxooooooxoooooox////////////",
        "////////////xooxxooxooxxooxooxxxxxooxooxxxxx////////////",
        "////////////xooxxooxooooooxooxooooooxoooooox////////////",
        "////////////xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx////////////"
    ]
);

#[component]
pub(crate) fn Email() -> impl IntoView {
    fn hostname() -> Option<String> {
        let window = window();
        let document = window.document()?;
        let location = document.location()?;
        location.hostname().ok()
    }
    let a_ref = NodeRef::<html::A>::new();

    let (display, set_display) = signal(Some("none"));
    Effect::new(move |_| {
        let Some(a) = a_ref.get() else {
            return;
        };
        let Some(hostname) = hostname() else {
            return;
        };
        let mut iter = hostname.rsplit('.');
        let Some(top_domain) = iter.next() else {
            return;
        };
        let Some(domain) = iter.next() else {
            return;
        };
        let mail = format!("marcus@{domain}.{top_domain}");
        a.set_href(&format!("mailto:{mail}"));
        if a.set_text(&mail).is_ok() {
            set_display.set(None);
        }
    });
    view! { <a node_ref=a_ref style:display=display></a> }
}

#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(PartialEq, Eq, Clone, Copy, Default)]
pub(crate) struct PersistentQrLogo {
    logo_animated: bool,
    log_written: bool,
}

pub(crate) struct Contact;

#[lazy_route]
impl LazyRoute for Contact {
    fn data() -> Self {
        Self
    }

    fn view(_this: Self) -> AnyView {
        #[cfg_attr(feature = "ssr", allow(unused))]
        let (show_canvas, set_show_canvas) = signal(Some("none"));
        #[cfg_attr(feature = "ssr", allow(unused))]
        let (vcard_href, set_vcard_href) = signal(None::<String>);
        let static_qr = NodeRef::<html::Img>::new();
        let canvas_ref = NodeRef::<html::Canvas>::new();
        let worm_canvas_ref = NodeRef::<html::Canvas>::new();
        let width = QR_CODE.width();
        let height = QR_CODE.height();
        let _static_assertion: () = {
            assert!(width == height);
        };
        #[cfg(not(feature = "ssr"))]
        Effect::new(move |_| {
            use crate::helpers::*;
            use leptos::logging::{log, warn};
            use qr_settings::*;
            use std::iter;
            use wasm_bindgen::JsCast;
            use web_sys::js_sys::Math;

            let worm_delay = 2.5f64;
            let animate_logo = {
                if let Some(c) = use_context::<ReadSignal<PersistentQrLogo>>() {
                    c.get_untracked().logo_animated == false
                } else {
                    true
                }
            };

            struct WormsState {
                qr_code: Vec<Vec<bool>>,
                width: usize,
                for_dark: Box<[Oco<'static, str>]>,
                for_light: Box<[Oco<'static, str>]>,
                canvas_context: web_sys::CanvasRenderingContext2d,
            }
            let Some(canvas) = canvas_ref.get() else {
                return;
            };
            let logo_shade_steps = (0..0xfe)
                .step_by(SHADE_STEP_SIZE.into())
                .chain(iter::once(0xff));
            let worm_shade_steps = (0..WORM_SHADE_STEPS)
                .chain(iter::once(WORM_SHADE_STEPS))
                .map(|s| s as f64 / WORM_SHADE_STEPS as f64)
                .map(|x| x * 255f64)
                .map(Math::round)
                .map(|x| Math::min(255f64, x))
                .map(|x| Math::max(0f64, x))
                .map(|x| x as u8);
            let [shade_to_dark, shade_to_light]: [Box<[Oco<'_, str>]>; _] = {
                let to_str = |x: u8| Oco::Counted(format!("rgb({x},{x},{x})").into());
                [
                    logo_shade_steps.clone().rev().map(to_str).collect(),
                    logo_shade_steps.map(to_str).collect(),
                ]
            };
            let [worm_for_dark, worm_for_light]: [Box<[Oco<'_, str>]>; _] = {
                let to_str = |x: u8| Oco::Counted(format!("rgb({x},{x},{x})").into());
                [
                    worm_shade_steps.clone().map(to_str).collect(),
                    worm_shade_steps.rev().map(to_str).collect(),
                ]
            };
            let mut state = {
                let Some(canvas) = worm_canvas_ref.get_untracked() else {
                    return;
                };
                let context = canvas
                    .get_context("2d")
                    .unwrap()
                    .unwrap()
                    .dyn_into::<web_sys::CanvasRenderingContext2d>()
                    .unwrap();
                WormsState {
                    qr_code: vec![vec![false; height]; width],
                    width,
                    for_dark: worm_for_dark,
                    for_light: worm_for_light,
                    canvas_context: context,
                }
            };
            let context = canvas
                .get_context("2d")
                .unwrap()
                .unwrap()
                .dyn_into::<web_sys::CanvasRenderingContext2d>()
                .unwrap();
            context.set_fill_style_str("#000000");
            context.fill_rect(
                0.0,
                0.0,
                u32::try_from(width).unwrap().into(),
                u32::try_from(height).unwrap().into(),
            );
            context.set_fill_style_str("#ffffff");
            for (pos, (pix, inv)) in QR_CODE.iter_merged().enumerate() {
                let x = (pos % width) as usize;
                let y = (pos / width) as usize;
                let orig = if animate_logo { pix } else { pix ^ inv };
                if orig {
                    context.fill_rect(x as f64, y as f64, 1.0, 1.0);
                }
                state.qr_code[x][y] = pix ^ inv;
                if animate_logo && inv {
                    let delay = std::time::Duration::from_secs_f64(
                        MIN_INITIAL_DELAY
                            + Math::random() * (MAX_INITIAL_DELAY - MIN_INITIAL_DELAY),
                    );
                    let interval = std::time::Duration::from_secs_f64(
                        MIN_SHADE_INTERVAL
                            + Math::random() * (MAX_SHADE_INTERVAL - MIN_SHADE_INTERVAL),
                    );
                    let canvas = canvas_ref.clone();
                    let r = if pix {
                        shade_to_dark.clone()
                    } else {
                        shade_to_light.clone()
                    };
                    set_scoped_timeout(delay, move || {
                        let Some(canvas) = canvas.get_untracked() else {
                            log!("Lost canvas");
                            return;
                        };
                        let context = canvas
                            .get_context("2d")
                            .unwrap()
                            .unwrap()
                            .dyn_into::<web_sys::CanvasRenderingContext2d>()
                            .unwrap();
                        r.into_iter()
                            .on_interval(interval, move |p| {
                                context.set_fill_style_str(&p);
                                context.fill_rect(x as f64, y as f64, 1.0, 1.0);
                            })
                            .into_scoped_timeout();
                    });
                }
            }
            set_show_canvas.set(None);
            if let Some(static_qr) = static_qr.get()
                && let Some(qr_parent) = static_qr.parent_node()
            {
                _ = qr_parent.remove_child(&static_qr);
            }
            if let Some(setter) = use_context::<WriteSignal<PersistentQrLogo>>() {
                set_scoped_timeout(
                    std::time::Duration::from_secs_f64(
                        MAX_INITIAL_DELAY
                            + MAX_SHADE_INTERVAL * (0xff as f64) / (SHADE_STEP_SIZE as f64),
                    ),
                    move || {
                        setter.update_untracked(|x| x.logo_animated = true);
                    },
                );
            }
            fn create_worm(state: WormsState) {
                let worm_length = MIN_WORM_LENGTH
                    + (Math::random() * (MAX_WORM_LENGTH - MIN_WORM_LENGTH) as f64) as usize;
                let mut x = (Math::random() * state.width as f64) as isize;
                let mut y = (Math::random() * state.width as f64) as isize;
                let mut angle = Math::random() * Math::PI.with(|x| x * 2f64);
                let mut worm_part_delay = 0f64;
                let Some(owner) = Owner::current() else {
                    return;
                };
                for _ in 0..worm_length {
                    x = Math::round(
                        x as f64
                            + Math::random()
                                * Math::max(Math::min(1f64, 2f64 * Math::cos(angle)), -1f64),
                    ) as isize;
                    y = Math::round(
                        y as f64
                            + Math::random()
                                * Math::max(Math::min(1f64, 2f64 * Math::sin(angle)), -1f64),
                    ) as isize;
                    const HALF_ANGLE: f64 = WORM_MOVE_ANGLE / 2.0;
                    angle = angle - HALF_ANGLE + Math::random() * WORM_MOVE_ANGLE;
                    worm_part_delay += WORM_MOVE_INTERVAL;
                    if std::cmp::min(x, y) < 0 || std::cmp::max(x, y) >= state.width as isize {
                        continue;
                    }

                    let light = state.qr_code[x as usize][y as usize];

                    let make_worm = if light {
                        state.for_light.clone()
                    } else {
                        state.for_dark.clone()
                    };
                    let context = state.canvas_context.clone();
                    owner.set_scoped_timeout(
                        std::time::Duration::from_secs_f64(worm_part_delay),
                        move || {
                            make_worm
                                .clone()
                                .into_iter()
                                .skip(1)
                                .chain(make_worm.into_iter().rev().skip(1))
                                .on_interval(
                                    std::time::Duration::from_secs_f64(WORM_SHADE_INTERVAL),
                                    move |shade| {
                                        context.set_fill_style_str(&shade);
                                        context.fill_rect(x as f64, y as f64, 1.0, 1.0);
                                    },
                                )
                                .into_scoped_timeout()
                        },
                    );
                }
                set_scoped_timeout(
                    std::time::Duration::from_secs_f64(
                        worm_part_delay
                            + MIN_WORM_INTERVAL
                            + Math::random() * (MAX_WORM_INTERVAL - MIN_WORM_INTERVAL),
                    ),
                    || request_scoped_animation_frame(|| create_worm(state)),
                );
            }
            set_scoped_timeout(
                std::time::Duration::from_secs_f64(
                    worm_delay
                        + MIN_WORM_INTERVAL
                        + Math::random() * (MAX_WORM_INTERVAL - MIN_WORM_INTERVAL),
                ),
                || {
                    set_scoped_timeout(
                        std::time::Duration::from_secs_f64(Math::random() * 5.0),
                        || {
                            if let Some(true) = use_context::<WriteSignal<PersistentQrLogo>>()
                                .map(|x| {
                                    x.try_maybe_update(|qr| {
                                        if qr.log_written {
                                            (false, false)
                                        } else {
                                            qr.log_written = true;
                                            (false, true)
                                        }
                                    })
                                })
                                .flatten()
                            {
                                warn!("Worm detected in browser");
                            }
                        },
                    );
                    create_worm(state);
                },
            );
            use base64::{Engine as _, engine::general_purpose::STANDARD};
            let content = STANDARD.encode(VCARD.as_bytes());
            set_vcard_href.set(Some(format!("data:text/vcard;base64,{content}")));
        });
        let original_qr_src = cfg_select! {
            feature = "static-qr" => {{
                use base64::{Engine as _, engine::general_purpose::STANDARD};
                let config = use_context::<LeptosOptions>().unwrap();
                assert_eq!("target/site", config.site_root.as_ref());
                let image = include_bytes!("../../target/site/qrcode.png");
                let content = STANDARD.encode(image);
                Some(format!("data:image/png;base64,{content}"))
            }}
            feature = "ssr" => {
                Some("qrcode.png")
            }
            _ => {
                None::<&'static str>
            }
        };

        view! {
            <div class="contact">
                <div
                    class="qr-code"
                    style:display="block"
                    style:box-sizing="border-box"
                    style:aspect-ratio="1/1"
                    style:width="calc(min(308px,100%))"
                    style:margin="0 auto 1em auto"
                >
                    <a download="Marcus Ofenhed.vcf" href=vcard_href>
                        <img node_ref=static_qr src=original_qr_src />
                        <NoScript>
                            <img class:fallback {..ImgDef()} src="qrlogo.png" />
                        </NoScript>
                        <NoWasm>
                            <img class:fallback {..ImgDef()} src="qrlogo.png" />
                        </NoWasm>
                        <div id="canvasHolder" style:display=show_canvas>
                            <canvas node_ref=canvas_ref width=width height=height />
                            <canvas node_ref=worm_canvas_ref width=width height=height />
                        </div>
                    </a>
                </div>
                <a href="https://linkedin.com/in/conditionraisemarcus">Marcus Ofenhed</a>
                <p>Senior IT Security Consultant</p>
                <Email />
            </div>
            <style {..custom_attribute("scoped", true)} media="print" nonce=use_nonce()>
                "div#canvasHolder>canvas:last-of-type{display:none}"
            </style>
        }
        .into_any()
    }
}

#[cfg(feature = "ssr")]
pub(crate) mod qr_generator {
    use super::*;
    use std::path::Path;
    #[derive(Debug, thiserror::Error)]
    pub enum QrGeneratorError {
        #[error("Qr image error: {0}")]
        ImageError(#[from] image::error::ImageError),
    }

    use image::*;
    pub fn save_qrcode(options: &LeptosOptions) -> Result<(), QrGeneratorError> {
        let width = QR_CODE.width();
        let height = QR_CODE.height();
        let mut original_image = ImageBuffer::from_pixel(
            width.try_into().unwrap(),
            height.try_into().unwrap(),
            image::Luma([0u8]),
        );
        let mut logo_image = ImageBuffer::from_pixel(
            width.try_into().unwrap(),
            height.try_into().unwrap(),
            image::Luma([0u8]),
        );
        for (pos, (pix, inv)) in QR_CODE.iter_merged().enumerate() {
            if pix {
                original_image.put_pixel(
                    (pos % width).try_into().unwrap(),
                    (pos / width).try_into().unwrap(),
                    image::Luma([0xff]),
                );
            }
            if pix ^ inv {
                logo_image.put_pixel(
                    (pos % width).try_into().unwrap(),
                    (pos / width).try_into().unwrap(),
                    image::Luma([0xff]),
                );
            }
        }
        original_image.save(Path::new(&*options.site_root).join("qrcode.png"))?;
        logo_image.save(Path::new(&*options.site_root).join("qrlogo.png"))?;
        Ok(())
    }
}
