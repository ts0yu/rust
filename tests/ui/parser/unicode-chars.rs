fn main() {
    let y = 0;
    //~^ ERROR unknown start of token: \u{37e}
    //~^^ HELP Unicode character ';' (Greek Question Mark) looks like ';' (Semicolon), but it is not
        let x = 0;
    //~^ ERROR unknown start of token: \u{a0}
    //~^^ NOTE character appears 3 more times
    //~^^^ HELP Unicode character ' ' (No-Break Space) looks like ' ' (Space), but it is not
}
