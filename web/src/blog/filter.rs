use std::{borrow::Cow, cell::LazyCell, iter, marker::PhantomData, str::FromStr, sync::LazyLock};

use leptos::{
    IntoView,
    children::{Children, ToChildren},
    either::{Either, EitherOr},
    oco::Oco,
    tachys::view::any_view::AnyView,
};
use leptos_router::{
    GeneratedRouteData, MatchInterface, MatchNestedRoutes, MatchParams, Method, PartialPathMatch,
    PathSegment, PossibleRouteMatch, SsrMode, any_nested_route::AnyNestedRoute,
    components::RouteChildren, static_routes::StaticRoute,
};
use strum::VariantArray;

pub trait Alternatives {
    fn list_alternatives() -> impl Iterator<Item = &'static str>;
}

pub trait FilterParam: AsRef<str> + VariantArray + FromStr {}
impl<T: AsRef<str> + VariantArray + FromStr> FilterParam for T {}

pub trait Filter {
    const NAME: &'static str = { panic!("Filter without name") };
    type PARAM: FilterParam;
    const OPTIONAL: bool = true;
}

pub trait LinkedFilters {
    const VALID: bool;
    type CURRENT: Filter;
    type NEXT: LinkedFilters;
}

//pub enum Route<F> {
//    pub child: AnyView,
//    pub _phantom: PhantomData<F>,
//}

pub struct FilterMatch<Filter> {
    filter: Filter,
}

pub trait MaybeNestedRoute {
    type NestedRoute;
    type View;

    fn route(&self) -> Option<&Self::NestedRoute>;
    fn route_mut(&mut self) -> Option<&mut Self::NestedRoute>;
    fn view(&self) -> Option<&Self::View>;
    fn view_mut(&mut self) -> Option<&mut Self::View>;
}

impl<R: MaybeNestedRoute> EitherOr for R {
    type Left = R::NestedRoute;
    type Right = R::View;

    fn either_or<FA, A, FB, B>(self, a: FA, b: FB) -> Either<A, B>
    where
        FA: FnOnce(Self::Left) -> A,
        FB: FnOnce(Self::Right) -> B,
    {
        todo!()
    }
}

#[component(transparent)]
pub fn RouteFilter<Child, MNR: MatchNestedRoutes = (), IV: IntoView = ()>(
    filter: impl Filter,
    children: FnOnce() -> Child,
) -> impl MatchNestedRoutes
where
    Child: EitherOr<Left = MatchNestedRoutes, Right = IV>,
{
}

//impl<F, V> MatchNestedRoutes for RouteFilter<F, V>
//where
//    F: Filter + 'static,
//    F::PARAM: FilterParam,
//{
//    fn optional(&self) -> bool {
//        F::OPTIONAL
//    }
//
//    type Data = ();
//
//    type Match = FilterMatch<F>;
//
//    fn match_nested<'a>(
//        &'a self,
//        path: &'a str,
//    ) -> (Option<(leptos_router::RouteMatchId, Self::Match)>, &'a str) {
//        (None, path)
//    }
//
//    fn generate_routes(&self) -> impl IntoIterator<Item = leptos_router::GeneratedRouteData> + '_ {
//        let no_match = if self.optional() { Some(vec![]) } else { None };
//        let child_routes = {
//            let (child_routes, default_route) =
//                match self.child.either_or(|x| x.generate_routes(), |_| ()) {
//                    Either::Left(l) => (Some(l.into_inner().generate_routes()), None),
//                    _ => (
//                        None,
//                        Some(GeneratedRouteData {
//                            segments: vec![],
//                            ssr_mode: SsrMode::Static(StaticRoute::new()),
//                            methods: [Method::Get].into(),
//                            regenerate: vec![],
//                        }),
//                    ),
//                };
//            child_routes
//                .into_iter()
//                .flatten()
//                .chain(default_route.into_iter())
//        };
//        let my_routes = no_match
//            .into_iter()
//            .chain(F::PARAM::VARIANTS.iter().map(|v| {
//                let name: Cow<'static, str> = Cow::Borrowed(v.as_ref().into());
//                vec![
//                    PathSegment::Static(Cow::Borrowed(F::NAME)),
//                    PathSegment::Static(name),
//                ]
//            }));
//        child_routes.flat_map(move |child| {
//            let segments = child.segments.clone();
//            my_routes.clone().map(move |mut route| {
//                route.append(&mut segments.clone());
//                GeneratedRouteData {
//                    segments: route,
//                    ssr_mode: child.ssr_mode.clone(),
//                    methods: child.methods.clone(),
//                    regenerate: child.regenerate.clone(),
//                }
//            })
//        })
//    }
//}
//
//impl<F: Filter> MatchInterface for FilterMatch<F>
//where
//    Self: 'static,
//{
//    type Child = Self;
//
//    fn as_id(&self) -> leptos_router::RouteMatchId {
//        todo!()
//    }
//
//    fn as_matched(&self) -> &str {
//        todo!()
//    }
//
//    fn into_view_and_child(self) -> (impl leptos_router::ChooseView, Option<Self::Child>) {
//        todo!()
//    }
//}
//
//impl<F: Filter> MatchParams for FilterMatch<F> {
//    fn to_params(&self) -> Vec<(Cow<'static, str>, String)> {
//        vec![]
//    }
//}
//
//impl<C, T: Filter + PossibleRouteMatch> PossibleRouteMatch for RouteFilter<C, T> {
//    fn optional(&self) -> bool {
//        T::OPTIONAL
//    }
//
//    fn test<'a>(&self, path: &'a str) -> Option<PartialPathMatch<'a>> {
//        let mut matched_len = 0;
//        let mut param_offset = 0;
//        let mut test = path.chars().skip_while(|x| {
//            if *x == '/' {
//                param_offset += 1;
//                true
//            } else {
//                false
//            }
//        });
//        let mut expected = T::NAME.chars();
//
//        loop {
//            match (test.next(), expected.next()) {
//                (Some(x), Some(y)) if x == y => {
//                    matched_len += 1;
//                }
//                (Some('/'), None) => break,
//                _ => {
//                    return None;
//                }
//            }
//        }
//
//        let Some(value) = self.0.test(&path[param_offset + matched_len..]) else {
//            return None;
//        };
//
//        let path_bytes: &[u8] = path.as_ref();
//
//        fn bytes_in(haystack: &[u8], needle: impl AsRef<[u8]>) -> Option<(usize, usize)> {
//            let needle = needle.as_ref();
//            haystack
//                .element_offset(&needle[0])
//                .map(|x| (x, needle.len()))
//        }
//        let (param_offset_bytes, _) =
//            bytes_in(path_bytes, &path[param_offset..param_offset]).expect("Is always a subset");
//
//        let (remaining_offset, _) = bytes_in(path_bytes, value.remaining())
//            .expect("PossibleRouteMatch will return a subset of the string");
//
//        let (submatch_offset, submatch_len) = bytes_in(path_bytes, value.matched())
//            .expect("PossibleRouteMatch will return a subset of the string");
//        unsafe {
//            return Some(PartialPathMatch::new(
//                str::from_utf8_unchecked(&path_bytes[remaining_offset..]),
//                vec![(
//                    Cow::Borrowed(T::NAME),
//                    path[submatch_offset..submatch_offset + submatch_len].to_string(),
//                )],
//                str::from_utf8_unchecked(
//                    &path_bytes[param_offset_bytes..submatch_offset + submatch_len],
//                ),
//            ));
//        }
//    }
//
//    fn generate_path(&self, path: &mut Vec<PathSegment>) {
//        path.push(PathSegment::Static(Cow::Borrowed(T::NAME)));
//        self.0.generate_path(path);
//    }
//}

//pub struct BlogFilter<'a> {
//    param: Oco<'a, str>,
//    value: Oco<'a, str>,
//}
//
//impl<'f> PossibleRouteMatch for BlogFilter<'f> {
//    fn optional(&self) -> bool {
//        true
//    }
//
//    fn test<'a>(&self, path: &'a str) -> Option<PartialPathMatch<'a>> {
//        let mut matched_len = 0;
//        let mut match_offset = 0;
//        let mut param_offset = 0;
//        let mut param_len = 0;
//        let mut test = path.chars();
//        let mut expected = self.param.chars();
//
//        if let Some('/') = test.next() {
//            param_offset += 1;
//            match_offset += 1;
//        }
//
//        loop {
//            match (test.next(), expected.next()) {
//                (Some(x), Some(y)) if x == y => {
//                    matched_len += 1;
//                    param_offset += 1;
//                }
//                (Some('/'), None) => {
//                    matched_len += 1;
//                    param_offset += 1;
//                    break;
//                }
//                _ => return None,
//            }
//        }
//        let param = test.take_while(|x| {
//            if *x != '/' {
//                matched_len += 1;
//                param_len += 1;
//                true
//            } else {
//                false
//            }
//        });
//
//        if param_len == 0 {
//            return None;
//        }
//
//        return Some(PartialPathMatch::new(
//            &path[match_offset + matched_len..],
//            vec![(
//                Cow::Owned(self.param.to_string()),
//                path[param_offset..param_offset + param_len].to_string(),
//            )],
//            &path[match_offset..match_offset + matched_len],
//        ));
//    }
//
//    fn generate_path(&self, path: &mut Vec<PathSegment>) {
//        let Some(ref value) = self.value else { return };
//        path.push(PathSegment::OptionalParam(Cow::Owned(format!(
//            "{}/{}",
//            self.param, value
//        ))));
//    }
//}
