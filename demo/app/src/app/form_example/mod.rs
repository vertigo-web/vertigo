use vertigo::{Computed, Value, bind, component, css, dom, transaction};

type ErrorLine = (Vec<String>, String);

#[derive(Clone)]
struct ValidatorError {
    message: Option<String>,
    errors: Vec<ErrorLine>,
}

trait FormNode<T: Clone + PartialEq + 'static> {
    fn result(&self, getter_name: impl Into<String>) -> Computed<Result<T, ValidatorError>>;
}

struct ErrorBuilder {
    errors: Vec<(Vec<String>, String)>,
}

impl ErrorBuilder {
    pub fn new() -> ErrorBuilder {
        ErrorBuilder { errors: Vec::new() }
    }

    pub fn add<T>(mut self, result: Result<T, ValidatorError>) -> Self {
        if let Err(err) = result {
            self.errors.extend(err.errors);
        }
        self
    }

    pub fn export(self) -> ValidatorError {
        ValidatorError {
            message: None,
            errors: self.errors,
        }
    }
}

// type Erro/

//........................................................................................
//........................................................................................
//........................................................................................

#[derive(Clone)]
struct FormValue<T: Clone + PartialEq + 'static> {
    text: Value<String>,
    value: Value<T>,
    // jedną stronę -->
    // w drugą stronę -->
}

impl<T: Clone + PartialEq + 'static> FormNode<T> for FormValue<T> {
    fn result(&self, _getter_name: impl Into<String>) -> Computed<Result<T, ValidatorError>> {
        todo!()
    }
}

//........................................................................................

#[derive(Clone)]
struct FormDate {
    day: FormValue<u8>,
    mounth: FormValue<u8>,
    year: FormValue<u16>,
    // error: Value<Option<String>>,
}

#[derive(Clone, PartialEq)]
struct FormDate1 {
    day: u8,
    mounth: u8,
    year: u16,
}

impl FormNode<FormDate1> for FormDate {
    fn result(
        &self,
        _getter_name: impl Into<String>,
    ) -> Computed<Result<FormDate1, ValidatorError>> {
        let ggg = self.clone();

        Computed::from(move |context| {
            let day = ggg.day.result("day").get(context);
            let mounth = ggg.mounth.result("mounth").get(context);
            let year = ggg.year.result("year").get(context);

            if let (Ok(day), Ok(mounth), Ok(year)) = (day.clone(), mounth.clone(), year.clone()) {
                return Ok(FormDate1 { day, mounth, year });
            }

            let errors = ErrorBuilder::new().add(day).add(mounth).add(year).export();
            Err(errors)
        })
    }
}

//........................................................................................

#[derive(Clone)]
struct FormModel {
    name: FormValue<String>,
    birth: FormDate,
    start_schools: FormDate,
}

#[derive(Clone, PartialEq)]
struct FormModel1 {
    name: String,
    birth: FormDate1,
    start_schools: FormDate1,
}

impl FormNode<FormModel1> for FormModel {
    fn result(
        &self,
        _getter_name: impl Into<String>,
    ) -> Computed<Result<FormModel1, ValidatorError>> {
        todo!()
    }
}

//........................................................................................

/*
    w input mogą występować jakieś stany "przejściowe" które nie są dozwolone, ale są
    konieczne zeby przejść z jednego dozwolonego stanu do innego


    zestawy róznych walidatorów jak ograć
    np, datę jak zrealizować ... ?

*/

#[component]
pub fn FormExample() {
    let name = Value::new("ddadas".to_string());
    let email = Value::new("".to_string());

    let on_submit = bind!(name, email, |_| {
        transaction(|context| {
            let name_val = name.get(context);
            let email_val = email.get(context);
            log::info!("Form submitted: name={}, email={}", name_val, email_val);
        });
    });

    let wrapper_css = css! {"
        border: 1px solid #ccc;
        padding: 20px;
        margin: 20px;
        border-radius: 8px;
        max-width: 400px;
        background-color: #f9f9f9;
        display: flex;
        flex-direction: column;
        gap: 15px;
    "};

    let label_css = css! {"
        display: flex;
        flex-direction: column;
        font-weight: bold;
        gap: 5px;
    "};

    let input_css = css! {"
        padding: 8px;
        border: 1px solid #999;
        border-radius: 4px;
        font-weight: normal;
    "};

    let button_css = css! {"
        padding: 10px;
        background-color: #007bff;
        color: white;
        border: none;
        border-radius: 4px;
        cursor: pointer;
        font-weight: bold;
        :hover {
            background-color: #0056b3;
        }
    "};

    dom! {
        <div css={wrapper_css}>
            <h2>"Form Example"</h2>
            <label css={label_css.clone()}>
                "Name:"
                <input
                    css={input_css.clone()}
                    value={&name}
                    on_input={bind!(name, |val| name.set(val))}
                    placeholder="Enter your name"
                />
            </label>

            <label css={label_css}>
                "Email:"
                <input
                    css={input_css}
                    value={&email}
                    on_input={bind!(email, |val| email.set(val))}
                    placeholder="Enter your email"
                />
            </label>

            <button css={button_css} on_click={on_submit}>
                "Submit"
            </button>

            <div>
                "Current name: " { name }
            </div>
            <div>
                "Current email: " { email }
            </div>
        </div>
    }
}
