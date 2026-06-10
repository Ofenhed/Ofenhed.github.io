use crate::blog::{
    BlogEntry,
    metadata::{Locale, Tag},
};
use chrono::DateTime;
use leptos::prelude::*;

#[component(transparent)]
pub fn Unremarkable() -> BlogEntry<Children> {
    let child = view! {
        <fieldset>
            <legend>Note</legend>
            "This is mostly the feedback I provided to Remarkable when I returned their "
            <a href="https://remarkable.com/products/remarkable-2">"Remarkable 2"</a>
            ". While it is no longer sold, I imagine that some of the critiques still apply or are worth considering before buying a Remarkable device. I received no reply to these critiques, but I had no issues whatsoever returning the device."
        </fieldset>
        <p>
            "I had such high hopes, and I was initially very happy with this product, I tried the writing and some sketching, looked through the (very few) settings, and the build quality is absolutely amazing."
        </p>
        <p>
            "The issues arose as soon as I tried to be productive with the device, which forced me to your cloud service. It's very interesting that you are charging for that \"service\", as I would gladly pay extra to not have it."
        </p>
        <p>"Some of the issues I've run into in the very short time I've tested the device:"</p>
        <ul>
            <li>
                "There is no way to have files on your device which are not synced to the cloud, unless you completely disconnect from the remarkable account. There goes most of my use cases, as no respectable customer"
                <ins>" of mine"</ins>" should be OK with that."
            </li>
            <li>
                "The Desktop client is simply a client to the cloud, it has no ability to sync files with the device, so without cloud you only have the 10.11.99.1 server to download files from, which only allows PDF downloads and no uploads. That means that without cloud, this device is extremely limited."
            </li>
            <li>
                "Images change (pretty significantly) from when I first create them to when I open them again. I know there is a (user developed) filter to fix this, but this is a big problem that should not be left to the community to fix for you, especially not with a preloaded library hack. This also should not actually solve the problem, as the lines look perfect when I first draw, it's just after having been saved and reopened that they look like trash."
            </li>
            <li>
                "I've noticed that cloud integration means that I can download a file and then upload another file, which creates many copies of the same file with some changes in each. This adds way more frustration than this issue should solve."
            </li>
            <li>
                "You have integration with other cloud services, but only through your own cloud service. This means that if I want integration with Drive/Dropbox/OneDrive, then I have to give "
                <b>"YOU"</b>" (not my device, but you as service providers) access to "
                <b>"MY ENTIRE ACCOUNTS"</b>"."
            </li>
            <li>
                "Only 4 digit screen lock? Something tells me there's also no encryption at rest... You're advertising this product for businesses, you really aren't acting like it."
            </li>
            <li>
                "I tried the screen sharing feature as well, but no invitation showed up on my desktop client. Restarted it with the same result."
            </li>
            <li>
                "Speaking of the desktop client, you should add your uninstaller to the Programs and Features list in Windows. Having to search for how to uninstall your program is never a good thing. Also, sign your binaries."
            </li>
            <li>
                "Only a Chrome plugin. Seriously? Take a look at "
                <a href="https://developer.mozilla.org/en-US/docs/Mozilla/Add-ons/WebExtensions/Build_a_cross_browser_extension">
                    "Mozilla's guide to building a cross browser extension"
                </a>"."
            </li>
            <li>
                "Why are some menus white on black, while most are black on white? White on black is a lot harder to read, why is there no accessibility option?"
            </li>
            <li>
                "You should provide hooks for developers, such as sync button press. One way would be to use systemd services."
                <ins>" The device is moddable, but it is very inconvenient."</ins>
            </li>
            <li>
                "You should not run everything as root. "<del>This</del><ins>
                    <a href="https://en.wikipedia.org/wiki/Principle_of_least_privilege">
                        "Principle of Least Privilege"
                    </a>
                </ins>
                " is pretty much lesson one in security, which makes me think that I don't want any sensitive data in your cloud."
                <ins>
                    " You may think me a stickler with this, but you are (again) advertising this as a business product, and it is to be expected that this device will be used to read untrusted PDF files, which is "
                    <a href="https://nvd.nist.gov/vuln/search#/nvd/home?keyword=PDF&resultType=records">
                        "notoriously hard to do securely"
                    </a>"."
                </ins>
            </li>
        </ul>
    };
    BlogEntry {
        uid: 1,
        publish_date: DateTime::parse_from_rfc3339("2022-01-20T15:38:00+01:00")
            .unwrap()
            .into(),
        last_updated: Some(
            DateTime::parse_from_rfc3339("2026-06-08T09:00:00+01:00")
                .unwrap()
                .into(),
        ),
        title: "(un)Remarkable 2",
        locale: Locale::EnglishSimplified.into(),
        tags: &[Tag::Review, Tag::Tech],
        children: Children::to_children(move || child),
    }
}
