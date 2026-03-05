use vertigo::{Computed, Value, bind, component, css, dom, transaction};

type ErrorLine = (Vec<String>, String);

#[derive(Clone)]
struct ValidatorError {
    message: Option<String>,
    errors: Vec<ErrorLine>,
}

impl ValidatorError {
    pub fn from_message(message: String) -> ValidatorError {
        ValidatorError {
            errors: Vec::new(), //vec![(vec![getter_name], message.clone())],
            message: Some(message),
        }
    }
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

    pub fn add<T>(mut self, getter_name: String, result: Result<T, ValidatorError>) -> Self {
        if let Err(err) = result {
            for (mut path, message) in err.errors {
                path.insert(0, getter_name.clone());
                self.errors.push((path, message));
            }

            if let Some(message) = err.message {
                self.errors.push((vec![getter_name], message));
            }
        }

        self
    }

    pub fn set_error(mut self, getter_name: String, message: String) -> Self {
        self.errors.push((vec![getter_name], message));
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
        getter_name: impl Into<String>,
    ) -> Computed<Result<FormDate1, ValidatorError>> {
        let ggg = self.clone();
        let getter_name = getter_name.into();

        Computed::from(move |context| {
            let day = ggg.day.result("day").get(context);
            let mounth = ggg.mounth.result("mounth").get(context);
            let year = ggg.year.result("year").get(context);

            if let (Ok(day), Ok(mounth), Ok(year)) = (day.clone(), mounth.clone(), year.clone()) {
                if mounth == 2 && day > 29 {
                    return Err(ValidatorError::from_message(
                        // getter_name.clone(),
                        "Luty nie moze mieć więcej niz 29 dni".into(),
                    ));
                }

                return Ok(FormDate1 { day, mounth, year });
            }

            let errors = ErrorBuilder::new()
                .add(getter_name.clone(), day)
                .add(getter_name.clone(), mounth)
                .add(getter_name.clone(), year)
                .export();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_date_errors() {
        let err_day = ValidatorError::from_message("invalid day".into());
        let err_month = ValidatorError::from_message("invalid month".into());

        // Simulate FormDate failing
        let date_errors = ErrorBuilder::new()
            .add("day".into(), Err::<(), _>(err_day))
            .add("month".into(), Err::<(), _>(err_month))
            .export();

        assert_eq!(date_errors.errors.len(), 2);
        assert_eq!(
            date_errors.errors[0],
            (vec!["day".to_string()], "invalid day".to_string())
        );
        assert_eq!(
            date_errors.errors[1],
            (vec!["month".to_string()], "invalid month".to_string())
        );
    }

    #[test]
    fn test_error_propagation() {
        let err_day = ValidatorError::from_message("invalid day".into());
        let err_month = ValidatorError::from_message("invalid month".into());

        // Simulate FormDate failing
        let date_errors = ErrorBuilder::new()
            .add("day".into(), Err::<(), _>(err_day))
            .add("month".into(), Err::<(), _>(err_month))
            .export();

        // Simulate FormModel failing because of birth (FormDate)
        let model_errors = ErrorBuilder::new()
            .add("birth".into(), Err::<(), _>(date_errors))
            .export();

        for (path, msg) in &model_errors.errors {
            println!("Path: {:?}, Msg: {}", path, msg);
        }

        // We expect:
        // ["birth", "day"] from err_day (nested in date_errors)
        // ["birth", "month"] from err_month (nested in date_errors)

        assert_eq!(model_errors.errors.len(), 2);
        assert_eq!(
            model_errors.errors[0],
            (
                vec!["birth".to_string(), "day".to_string()],
                "invalid day".to_string()
            )
        );
        assert_eq!(
            model_errors.errors[1],
            (
                vec!["birth".to_string(), "month".to_string()],
                "invalid month".to_string()
            )
        );
    }
}
