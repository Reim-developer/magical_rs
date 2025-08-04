use magical_rs::magical::magic_custom::{CustomMatchRules, MagicCustom, match_types_custom};

/*
* First. You should to define
* your own enum. It can be any
* type you want, like
* CuteCatGirl or CatChan.
*/
#[derive(Clone, Copy, Debug)]
enum CuteGirlKind {
    ShoujoFile,
    MikotoChanFile,
    UnknownFallback,
}

/*
* Simple function to detect file with your customize
* rule
*/
fn my_custom_magic() {
    /*
     * Define your rule, signature and offset here.
     * If you don't need the function pointer
     * to handle it separately like the next
     * function, leave it as default.
     */
    let rule: MagicCustom<CuteGirlKind> = MagicCustom {
        signatures: &[b"MagicalGirl"],
        offsets: &[0],
        max_bytes_read: 69,
        kind: CuteGirlKind::ShoujoFile,
        rules: CustomMatchRules::Default,
    };

    let my_bytes = b"MagicalGirl";
    /*
     * You will need to pass 3 parameters
     * to the function parameters.
     *
     * Specifically:
     * Bytes: Bytes of your file/or something. As long
     * as is bytes.
     *
     * Rules: Rules you just defined above
     *
     * Fallback: If None of cases match, what does it return?
     * Pass that to the function. And it must be
     * something in your enum.
     */
    let result = match_types_custom(my_bytes, &[rule], CuteGirlKind::UnknownFallback);

    println!("{result:?}"); /* ShoujoFile */
}

/*
* Simple function to detect file with your customize rule.
* But this time. We'll add a litle magic to it.
*/
fn my_custom_magic_with_fn() {
    /*
     * This is a magic function for us
     * to recognize a cute girl Mikoto-chan.
     */
    fn detect_mikoto_chan(bytes: &[u8]) -> bool {
        bytes.starts_with(b"MikotoChan")
    }

    let rule: MagicCustom<CuteGirlKind> = MagicCustom {
        /* Leave signatures & offsets blank.
         * To activate magic. ~ nia!
         */
        signatures: &[],
        offsets: &[],
        max_bytes_read: 69,
        kind: CuteGirlKind::MikotoChanFile,
        rules: CustomMatchRules::WithFn(detect_mikoto_chan),
    };

    let my_bytes = b"MikotoChan";
    let result = match_types_custom(my_bytes, &[rule], CuteGirlKind::UnknownFallback);

    println!("{result:?}"); /* MikotoChanFile */
}

/*
* Finnaly. We'll use magic to make things
* neater
*/
fn my_magic_detect() {
    /*
     * This is a magic function for us
     * to recognize a cute girl Mikoto-chan.
     */
    fn detect_mikoto_chan(bytes: &[u8]) -> bool {
        bytes.starts_with(b"MikotoChan")
    }

    /*
     * We can do this indefinitely as long.
     * If you RAM can handle it.
     */
    let rule: &[MagicCustom<CuteGirlKind>] = &[
        MagicCustom {
            /* Leave signatures & offsets blank.
             * To activate magic. ~ nia!
             */
            signatures: &[],
            offsets: &[],
            max_bytes_read: 69,
            kind: CuteGirlKind::MikotoChanFile,
            rules: CustomMatchRules::WithFn(detect_mikoto_chan),
        },
        MagicCustom {
            signatures: &[b"MagicalGirl"],
            offsets: &[0],
            max_bytes_read: 69,
            kind: CuteGirlKind::ShoujoFile,
            rules: CustomMatchRules::Default,
        },
    ];

    let my_bytes = b"MikotoChan";
    let my_bytes_2 = b"MagicalGirl";

    let result_1 = match_types_custom(my_bytes, rule, CuteGirlKind::UnknownFallback);
    let result_2 = match_types_custom(my_bytes_2, rule, CuteGirlKind::UnknownFallback);

    println!("{result_1:?}"); /* MikotoChanFile */
    println!("{result_2:?}"); /* MagicalGirl */
}

fn main() {
    my_custom_magic();
    my_custom_magic_with_fn();
    my_magic_detect();
}
