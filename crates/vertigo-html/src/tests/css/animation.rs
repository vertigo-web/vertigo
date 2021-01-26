use crate::css_fn;

use super::utils::*;

#[test]
fn animation_css() {
    css_fn! { css_factory, {
        animation: 5s linear 2s infinite alternate {
            0%   {background-color: red; left: 0px; top: 0px;}
            25%  {background-color: yellow; left: 200px; top: 0px;}
            50%  {background-color: blue; left: 200px; top: 200px;}
            75%  {background-color: green; left: 0px; top: 200px;}
            100% {background-color: red; left: 0px; top: 0px;}
        };
    }}

    let value = css_factory();

    assert_eq!(get_s(&value), "animation: 5s linear 2s infinite alternate { 0% { background-color: red;
left: 0px;
top: 0px; }
25% { background-color: yellow;
left: 200px;
top: 0px; }
50% { background-color: blue;
left: 200px;
top: 200px; }
75% { background-color: green;
left: 0px;
top: 200px; }
100% { background-color: red;
left: 0px;
top: 0px; } };")
}
