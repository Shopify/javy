pub struct JsModule {
    js_src: String,
}

impl JsModule {
    pub fn new(js_src: &str) -> JsModule {
        Self { js_src: js_src.to_string() }
    }

    pub fn to_wat(&self) -> String {
        let mut tera = tera::Tera::default();
        tera.add_raw_template(
            "js_module.wat",
            std::include_str!("templates/js_module.wat"),
        )
        .unwrap();

        let js_bytes = self.js_src.as_bytes();
        let js_bytes_escaped: String = js_bytes.iter().map(|b| format!("\\{:02X?}", b)).collect();

        let mut context = tera::Context::new();
        context.insert("js_string_length_bytes", &js_bytes.len());
        context.insert("js_string_bytes", &js_bytes_escaped);
        tera.render("js_module.wat", &context).unwrap()
    }
}

// Not sure how to get this test to run, cargo test is not picking it up
// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_js_module_to_wat() {
//         let js_src = "console.log('hello world');";
//         let js_module = JsModule::new(js_src);
//         let wat = js_module.to_wat();
//         assert!(wat.contains(js_src));
//     }
// }