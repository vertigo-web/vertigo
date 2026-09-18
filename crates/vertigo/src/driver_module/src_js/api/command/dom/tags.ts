// Tag-name handling for the command applier (`dom.ts`).

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

export const createElement = (name: string): Element => {
    if (SVG_TAGS.has(name)) {
        return document.createElementNS("http://www.w3.org/2000/svg", name.replace("svg:", ""));
    } else {
        return document.createElement(name);
    }
}
