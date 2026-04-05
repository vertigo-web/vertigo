use super::utils::*;
use crate::css;

// Make crate available by its name for css macro
use crate as vertigo;

#[test]
fn test_animation() {
    let value = css! {"
        animation: 5s linear 2s infinite alternate {
            0%   {background-color: red; left: 0px; top: 0px;}
            25%  {background-color: yellow; left: 200px; top: 0px;}
            50%  {background-color: blue; left: 200px; top: 200px;}
            75%  {background-color: green; left: 0px; top: 200px;}
            100% {background-color: red; left: 0px; top: 0px;}
        };
    "};

    assert_eq!(
        get_s(&value),
        "animation: 5s linear 2s infinite alternate { 0% { background-color: red;
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
top: 0px; } };"
    )
}

#[test]
fn test_animation_with_rgba_and_comments() {
    let value = css! {"
        color: red; /* It's good here */
        animation: 2s ease-out forwards {
            0% {
                background-color: rgba(255, 255, 0, 1); /* Parse error here */
            }
            20% {
                background-color: rgba(255, 255, 0, 0.8);
            }
            100% {
                background-color: rgba(255, 255, 0, 0);
            }
        };
    "};

    assert_eq!(
        get_s(&value),
        "color: red;\nanimation: 2s ease-out forwards { 0% { background-color: rgba(255, 255, 0, 1); }
20% { background-color: rgba(255, 255, 0, 0.8); }
100% { background-color: rgba(255, 255, 0, 0); } };"
    )
}
