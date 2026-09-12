// Tag-name handling, shared by the command applier (`dom.ts`) and the hydration matcher
// (`hydration.ts`).

// Workaround, remove when https://github.com/vertigo-web/vertigo/issues/539 is done.
const SVG_TAGS = new Set([
    "animate", "animateMotion", "animateTransform", "circle", "clipPath", "defs",
    "desc", "discard", "ellipse", "feBlend", "feColorMatrix", "feComponentTransfer",
    "feComposite", "feConvolveMatrix", "feDiffuseLighting", "feDisplacementMap",
    "feDistantLight", "feDropShadow", "feFlood", "feFuncA", "feFuncB", "feFuncG",
    "feFuncR", "feGaussianBlur", "feImage", "feMerge", "feMergeNode", "feMorphology",
    "feOffset", "fePointLight", "feSpecularLighting", "feSpotLight", "feTile",
    "feTurbulence", "filter", "foreignObject", "g", "hatch", "hatchpath", "image",
    "line", "linearGradient", "marker", "mask", "metadata", "mpath", "path", "pattern",
    "polygon", "polyline", "radialGradient", "rect", "set", "stop", "svg", "switch",
    "symbol", "text", "textPath", "tspan", "use", "view",
    "svg:a", "svg:title", "svg:desc", "svg:script", "svg:style"
]);

/// The name an element created from `name` reports as its `tagName`.
///
/// HTML elements uppercase it; SVG ones keep the case they were created with, and the HTML
/// parser applies the same adjustment when it reads server-rendered markup - so `<svg>` comes
/// back as "svg" and `<linearGradient>` as "linearGradient", never "SVG" or "LINEARGRADIENT".
/// Hydration compares against this rather than blanket-uppercasing.
export const expectedTagName = (name: string): string => {
    if (SVG_TAGS.has(name)) {
        return name.replace("svg:", "");
    }

    return name.toUpperCase();
};

export const createElement = (name: string): Element => {
    if (SVG_TAGS.has(name)) {
        return document.createElementNS("http://www.w3.org/2000/svg", name.replace("svg:", ""));
    } else {
        return document.createElement(name);
    }
}
