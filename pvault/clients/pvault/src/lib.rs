mod code_gen {
    use anchor_lang::AnchorAccount;
    anchor_lang::declare_program!(pvault);
}

pub use code_gen::pvault;
