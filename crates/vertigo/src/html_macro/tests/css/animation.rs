use crate::css;
use super::utils::*;

// Make crate available by its name for css macro
use crate as vertigo;

#[test]
fn test_animation() {
    let value = css!("
        animation: 5s linear 2s infinite alternate {
            0%   {background-color: red; left: 0px; top: 0px;}
            25%  {background-color: yellow; left: 200px; top: 0px;}
            50%  {background-color: blue; left: 200px; top: 200px;}
            75%  {background-color: green; left: 0px; top: 200px;}
            100% {background-color: red; left: 0px; top: 0px;}
        };
    ");

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
