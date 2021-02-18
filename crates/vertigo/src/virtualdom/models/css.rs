pub enum CssGroup {
    CssStatic {
        value: &'static str,                    //&str -- moze zachowywac sie jako id po ktorym odnajdujemy interesujaca regule
    },
    CssDynamic {
        value: String,                          //w tym wypadku String, jest kluczem do hashmapy z wynikowa nazwa klasy
    }
}

pub struct Css {
    pub groups: Vec<CssGroup>,
}

impl Default for Css {
    fn default() -> Self {
        Self {
            groups: Vec::new()
        }
    }
}

impl Css {
    pub fn str(value: &'static str) -> Self {
        Self {
            groups: vec!(CssGroup::CssStatic {
                value
            })
        }
    }

    pub fn string(value: String) -> Self {
        Self {
            groups: vec!(CssGroup::CssDynamic {
                value
            })
        }
    }

    pub fn push_str(mut self, value: &'static str) -> Self {
        self.groups.push(CssGroup::CssStatic {
            value
        });
        self
    }

    pub fn push_string(&mut self, value: String) {
        self.groups.push(CssGroup::CssDynamic {
            value
        })
    }
}
