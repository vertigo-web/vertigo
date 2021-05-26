use vertigo::VDomElement;
use vertigo::VDomText;

use crate::html;

#[test]
fn style_basic2() {
    let dom1 = html!{
        <p>
            "text1"
            <span>"mokate"</span>
            "text2"
        </p>
    };


    let dom2 = VDomElement::build("p")
        .children(
            vec!(
                VDomText::new("text1").into(),
                VDomElement::build("span")
                    .children(vec!(
                        VDomText::new("mokate").into(),
                    ))
                    .into(),
                VDomText::new("text2").into()
            )
        )
    ;

    assert_eq!(
        format!("{:?}", dom1),
        format!("{:?}", dom2)
    );
}


#[test]
fn test_p() {
    let dom1 = html!{
        <p>"text33333 dsadsada ^^ && $$$ fff"</p>
    };


    let dom2 = VDomElement::build("p")
        .children(
            vec!(
                VDomText::new("text33333 dsadsada ^^ && $$$ fff")
                    .into()
            )
        )
    ;

    assert_eq!(
        format!("{:?}", dom1),
        format!("{:?}", dom2)
    );
}
