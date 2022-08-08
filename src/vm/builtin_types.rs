impl Globals {
    pub fn init_types(&mut self) {
        TypeBuilder::new(ValueType::Array)
            .add_method(
                &mut Globals,
                method! {
                    push(self: mut Array, el: Int) => self.push(el)
                },
            )
            .finish_type();
    }
}
