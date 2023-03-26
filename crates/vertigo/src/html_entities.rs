#![allow(non_upper_case_globals)]
//! Constants of unicode signs labeled with HTML entities names
//!
//! ```rust
//! use vertigo::{dom, DomNode, html_entities::*};
//!
//! fn render() -> DomNode {
//!    dom! {
//!       <div>{larr} " Back"</div>
//!    }
//! }
//! ```

/// ampersand
pub const amp: char = '&';
/// less than
pub const lt: char = '<';
/// greater than
pub const gt: char = '>';
/// no-break space = non-breaking space
pub const nbsp: char = ' ';
/// inverted exclamation mark
pub const iexcl: char = '¡';
/// cent sign
pub const cent: char = '¢';
/// pound sign
pub const pound: char = '£';
/// currency sign
pub const curren: char = '¤';
/// yen sign = yuan sign
pub const yen: char = '¥';
/// broken bar = broken vertical bar
pub const brvbar: char = '¦';
/// section sign
pub const sect: char = '§';
/// diaeresis = spacing diaeresis
pub const uml: char = '¨';
/// copyright sign
pub const copy: char = '©';
/// feminine ordinal indicator
pub const ordf: char = 'ª';
/// left-pointing double angle quotation mark = left pointing guillemet
pub const laquo: char = '«';
/// not sign
pub const not: char = '¬';
/// soft hyphen = discretionary hyphen
pub const shy: char = '\u{AD}';
/// registered sign = registered trade mark sign
pub const reg: char = '®';
/// macron = spacing macron = overline = APL overbar
pub const macr: char = '¯';
/// degree sign
pub const deg: char = '°';
/// plus-minus sign = plus-or-minus sign
pub const plusmn: char = '±';
/// superscript two = superscript digit two = squared
pub const sup2: char = '²';
/// superscript three = superscript digit three = cubed
pub const sup3: char = '³';
/// acute accent = spacing acute
pub const acute: char = '´';
/// micro sign
pub const micro: char = 'µ';
/// pilcrow sign = paragraph sign
pub const para: char = '¶';
/// middle dot = Georgian comma = Greek middle dot
pub const middot: char = '·';
/// cedilla = spacing cedilla
pub const cedil: char = '¸';
/// superscript one = superscript digit one
pub const sup1: char = '¹';
/// masculine ordinal indicator
pub const ordm: char = 'º';
/// right-pointing double angle quotation mark = right pointing guillemet
pub const raquo: char = '»';
/// vulgar fraction one quarter = fraction one quarter
pub const frac14: char = '¼';
/// vulgar fraction one half = fraction one half
pub const frac12: char = '½';
/// vulgar fraction three quarters = fraction three quarters
pub const frac34: char = '¾';
/// inverted question mark = turned question mark
pub const iquest: char = '¿';
/// latin capital letter A with grave = latin capital letter A grave
pub const Agrave: char = 'À';
/// latin capital letter A with acute
pub const Aacute: char = 'Á';
/// latin capital letter A with circumflex
pub const Acirc: char = 'Â';
/// latin capital letter A with tilde
pub const Atilde: char = 'Ã';
/// latin capital letter A with diaeresis
pub const Auml: char = 'Ä';
/// latin capital letter A with ring above = latin capital letter A ring
pub const Aring: char = 'Å';
/// latin capital letter AE = latin capital ligature AE
pub const AElig: char = 'Æ';
/// latin capital letter C with cedilla
pub const Ccedil: char = 'Ç';
/// latin capital letter E with grave
pub const Egrave: char = 'È';
/// latin capital letter E with acute
pub const Eacute: char = 'É';
/// latin capital letter E with circumflex
pub const Ecirc: char = 'Ê';
/// latin capital letter E with diaeresis
pub const Euml: char = 'Ë';
/// latin capital letter I with grave
pub const Igrave: char = 'Ì';
/// latin capital letter I with acute
pub const Iacute: char = 'Í';
/// latin capital letter I with circumflex
pub const Icirc: char = 'Î';
/// latin capital letter I with diaeresis
pub const Iuml: char = 'Ï';
/// latin capital letter ETH
pub const ETH: char = 'Ð';
/// latin capital letter N with tilde
pub const Ntilde: char = 'Ñ';
/// latin capital letter O with grave
pub const Ograve: char = 'Ò';
/// latin capital letter O with acute
pub const Oacute: char = 'Ó';
/// latin capital letter O with circumflex
pub const Ocirc: char = 'Ô';
/// latin capital letter O with tilde
pub const Otilde: char = 'Õ';
/// latin capital letter O with diaeresis
pub const Ouml: char = 'Ö';
/// multiplication sign
pub const times: char = '×';
/// latin capital letter O with stroke = latin capital letter O slash
pub const Oslash: char = 'Ø';
/// latin capital letter U with grave
pub const Ugrave: char = 'Ù';
/// latin capital letter U with acute
pub const Uacute: char = 'Ú';
/// latin capital letter U with circumflex
pub const Ucirc: char = 'Û';
/// latin capital letter U with diaeresis
pub const Uuml: char = 'Ü';
/// latin capital letter Y with acute
pub const Yacute: char = 'Ý';
/// latin capital letter THORN
pub const THORN: char = 'Þ';
/// latin small letter sharp s = ess-zed
pub const szlig: char = 'ß';
/// latin small letter a with grave = latin small letter a grave
pub const agrave: char = 'à';
/// latin small letter a with acute
pub const aacute: char = 'á';
/// latin small letter a with circumflex
pub const acirc: char = 'â';
/// latin small letter a with tilde
pub const atilde: char = 'ã';
/// latin small letter a with diaeresis
pub const auml: char = 'ä';
/// latin small letter a with ring above = latin small letter a ring
pub const aring: char = 'å';
/// latin small letter ae = latin small ligature ae
pub const aelig: char = 'æ';
/// latin small letter c with cedilla
pub const ccedil: char = 'ç';
/// latin small letter e with grave
pub const egrave: char = 'è';
/// latin small letter e with acute
pub const eacute: char = 'é';
/// latin small letter e with circumflex
pub const ecirc: char = 'ê';
/// latin small letter e with diaeresis
pub const euml: char = 'ë';
/// latin small letter i with grave
pub const igrave: char = 'ì';
/// latin small letter i with acute
pub const iacute: char = 'í';
/// latin small letter i with circumflex
pub const icirc: char = 'î';
/// latin small letter i with diaeresis
pub const iuml: char = 'ï';
/// latin small letter eth
pub const eth: char = 'ð';
/// latin small letter n with tilde
pub const ntilde: char = 'ñ';
/// latin small letter o with grave
pub const ograve: char = 'ò';
/// latin small letter o with acute
pub const oacute: char = 'ó';
/// latin small letter o with circumflex
pub const ocirc: char = 'ô';
/// latin small letter o with tilde
pub const otilde: char = 'õ';
/// latin small letter o with diaeresis
pub const ouml: char = 'ö';
/// division sign
pub const divide: char = '÷';
/// latin small letter o with stroke = latin small letter o slash
pub const oslash: char = 'ø';
/// latin small letter u with grave
pub const ugrave: char = 'ù';
/// latin small letter u with acute
pub const uacute: char = 'ú';
/// latin small letter u with circumflex
pub const ucirc: char = 'û';
/// latin small letter u with diaeresis
pub const uuml: char = 'ü';
/// latin small letter y with acute
pub const yacute: char = 'ý';
/// latin small letter thorn
pub const thorn: char = 'þ';
/// latin small letter y with diaeresis
pub const yuml: char = 'ÿ';
/// latin small f with hook = function = florin
pub const fnof: char = 'ƒ';
/// greek capital letter alpha
pub const Alpha: char = 'Α';
/// greek capital letter beta
pub const Beta: char = 'Β';
/// greek capital letter gamma
pub const Gamma: char = 'Γ';
/// greek capital letter delta
pub const Delta: char = 'Δ';
/// greek capital letter epsilon
pub const Epsilon: char = 'Ε';
/// greek capital letter zeta
pub const Zeta: char = 'Ζ';
/// greek capital letter eta
pub const Eta: char = 'Η';
/// greek capital letter theta
pub const Theta: char = 'Θ';
/// greek capital letter iota
pub const Iota: char = 'Ι';
/// greek capital letter kappa
pub const Kappa: char = 'Κ';
/// greek capital letter lambda
pub const Lambda: char = 'Λ';
/// greek capital letter mu
pub const Mu: char = 'Μ';
/// greek capital letter nu
pub const Nu: char = 'Ν';
/// greek capital letter xi
pub const Xi: char = 'Ξ';
/// greek capital letter omicron
pub const Omicron: char = 'Ο';
/// greek capital letter pi
pub const Pi: char = 'Π';
/// greek capital letter rho
pub const Rho: char = 'Ρ';
/// greek capital letter sigma
pub const Sigma: char = 'Σ';
/// greek capital letter tau
pub const Tau: char = 'Τ';
/// greek capital letter upsilon
pub const Upsilon: char = 'Υ';
/// greek capital letter phi
pub const Phi: char = 'Φ';
/// greek capital letter chi
pub const Chi: char = 'Χ';
/// greek capital letter psi
pub const Psi: char = 'Ψ';
/// greek capital letter omega
pub const Omega: char = 'Ω';
/// greek smal letter alpha
pub const alpha: char = 'α';
/// greek smal letter beta
pub const beta: char = 'β';
/// greek smal letter gamma
pub const gamma: char = 'γ';
/// greek smal letter delta
pub const delta: char = 'δ';
/// greek smal letter epsilon
pub const epsilon: char = 'ε';
/// greek smal letter zeta
pub const zeta: char = 'ζ';
/// greek smal letter eta
pub const eta: char = 'η';
/// greek smal letter theta
pub const theta: char = 'θ';
/// greek smal letter iota
pub const iota: char = 'ι';
/// greek smal letter kappa
pub const kappa: char = 'κ';
/// greek smal letter lambda
pub const lambda: char = 'λ';
/// greek smal letter mu
pub const mu: char = 'μ';
/// greek smal letter nu
pub const nu: char = 'ν';
/// greek smal letter xi
pub const xi: char = 'ξ';
/// greek smal letter omicron
pub const omicron: char = 'ο';
/// greek smal letter pi
pub const pi: char = 'π';
/// greek smal letter rho
pub const rho: char = 'ρ';
/// greek smal letter final sigma
pub const sigmaf: char = 'ς';
/// greek smal letter sigma
pub const sigma: char = 'σ';
/// greek smal letter tau
pub const tau: char = 'τ';
/// greek smal letter upsilon
pub const upsilon: char = 'υ';
/// greek smal letter phi
pub const phi: char = 'φ';
/// greek smal letter chi
pub const chi: char = 'χ';
/// greek smal letter psi
pub const psi: char = 'ψ';
/// greek smal letter omega
pub const omega: char = 'ω';
/// greek smal letter theta symbol
pub const thetasym: char = 'ϑ';
/// greek upsilon with hook symbol
pub const upsih: char = 'ϒ';
/// greek pi symbol
pub const piv: char = 'ϖ';

/// bullet = black small circle
pub const bull: char = '•';
/// horizontal ellipsis = three dot leader
pub const hellip: char = '…';
/// prime = minutes = feet
pub const prime: char = '′';
/// double prime = seconds = inches
pub const Prime: char = '″';
/// overline = spacing overscore
pub const oline: char = '‾';
/// fraction slash
pub const frasl: char = '⁄';
/// script capital P = power set = Weierstrass p
pub const weierp: char = '℘';
/// blackletter capital I = imaginary part
pub const image: char = 'ℑ';
/// blackletter capital R = real part symbol
pub const real: char = 'ℜ';
/// trade mark sign
pub const trade: char = '™';
/// alef symbol = first transfinite cardinal
pub const alefsym: char = 'ℵ';
/// leftwards arrow
pub const larr: char = '←';
/// upwards arrow
pub const uarr: char = '↑';
/// rightwards arrow
pub const rarr: char = '→';
/// downwards arrow
pub const darr: char = '↓';
/// left right arrow
pub const harr: char = '↔';
/// downwards arrow with corner leftwards = carriage return
pub const crarr: char = '↵';
/// leftwards double arrow
pub const lArr: char = '⇐';
/// upwards double arrow
pub const uArr: char = '⇑';
/// rightwards double arrow
pub const rArr: char = '⇒';
/// downwards double arrow
pub const dArr: char = '⇓';
/// left right double arrow
pub const hArr: char = '⇔';
/// for all
pub const forall: char = '∀';
/// partial differential
pub const part: char = '∂';
/// there exists
pub const exist: char = '∃';
/// empty set = null set = diameter
pub const empty: char = '∅';
/// nabla = backward difference
pub const nabla: char = '∇';
/// element of
pub const isin: char = '∈';
/// not an element of
pub const notin: char = '∉';
/// contains as member
pub const ni: char = '∋';
/// n-ary product = product sign
pub const prod: char = '∏';
/// n-ary sumation
pub const sum: char = '∑';
/// minus sign
pub const minus: char = '−';
/// asterisk operator
pub const lowast: char = '∗';
/// square root = radical sign
pub const radic: char = '√';
/// proportional to
pub const prop: char = '∝';
/// infinity
pub const infin: char = '∞';
/// angle
pub const ang: char = '∠';
/// logical and = wedge
pub const and: char = '∧';
/// logical or = vee
pub const or: char = '∨';
/// intersection = cap
pub const cap: char = '∩';
/// union = cup
pub const cup: char = '∪';
/// integral
pub const int: char = '∫';
/// therefore
pub const there4: char = '∴';
/// tilde operator = varies with = similar to
pub const sim: char = '∼';
/// approximately equal to
pub const cong: char = '≅';
/// almost equal to = asymptotic to
pub const asymp: char = '≈';
/// not equal to
pub const ne: char = '≠';
/// identical to
pub const equiv: char = '≡';
/// less-than or equal to
pub const le: char = '≤';
/// greater-than or equal to
pub const ge: char = '≥';
/// subset of
pub const sub: char = '⊂';
/// superset of
pub const sup: char = '⊃';
/// not a subset of
pub const nsub: char = '⊄';
/// subset of or equal to
pub const sube: char = '⊆';
/// superset of or equal to
pub const supe: char = '⊇';
/// circled plus = direct sum
pub const oplus: char = '⊕';
/// circled times = vector product
pub const otimes: char = '⊗';
/// up tack = orthogonal to = perpendicular
pub const perp: char = '⊥';
/// dot operator
pub const sdot: char = '⋅';
/// left ceiling = APL upstile
pub const lceil: char = '⌈';
/// right ceiling
pub const rceil: char = '⌉';
/// left floor = APL downstile
pub const lfloor: char = '⌊';
/// right floor
pub const rfloor: char = '⌋';
/// left-pointing angle bracket = bra
pub const lang: char = '〈';
/// right-pointing angle bracket = ket
pub const rang: char = '〉';
/// lozenge
pub const loz: char = '◊';
/// black spade suit
pub const spades: char = '♠';
/// black club suit = shamrock
pub const clubs: char = '♣';
/// black heart suit = valentine
pub const hearts: char = '♥';
/// black diamond suit
pub const diams: char = '♦';
