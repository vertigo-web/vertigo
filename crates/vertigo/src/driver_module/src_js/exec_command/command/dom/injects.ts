import { AppLocation } from "../../location/AppLocation";

export function hydrate_link(node: Element, appLocation: AppLocation) {
    node.addEventListener('click', (e) => {
        let href = node.getAttribute('href');
        if (href === null) {
            return;
        }

        if (href.startsWith('#') || href.startsWith('http://') || href.startsWith('https://') || href.startsWith('//')) {
            return;
        }

        e.preventDefault();
        appLocation.set('History', 'Push', href);
        window.scrollTo(0, 0);
    })
}
