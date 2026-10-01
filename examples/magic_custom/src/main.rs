use magical_rs::magical::magic_custom::{CustomMatchRules, MagicCustom, match_types_custom};
use v_2_0_0::magic_custom_macro;

pub mod v_2_0_0 {
    pub mod magic_custom_macro;
}

/*
* First. You should to define
* your own enum. It can be any
* type you want, like
* CuteCatGirl or CatChan.
*/
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

/*
* The same three rules as `my_magic_detect`, written with
* `magic_rules!`.
*
* This is the form to reach for. The three functions above each spell out a
* five-field `MagicCustom` literal, and two of the five fields are noise in every
* one of them: a byte rule writes `offsets: &[0]` every time, and a predicate
* rule writes `signatures: &[]` and `offsets: &[]` every time. The macro is sugar
* for that literal and nothing else, so the answers below are the answers above --
* the last two lines are asserted rather than printed so a reader who runs this
* finds out.
*/
fn my_magic_detect_with_the_macro() {
    use magical_rs::magic_rules;

    fn detect_mikoto_chan(bytes: &[u8]) -> bool {
        bytes.starts_with(b"MikotoChan")
    }

    static RULES: &[MagicCustom<CuteGirlKind>] = magic_rules![
        /*
         * A predicate rather than bytes. The macro leaves `signatures` and
         * `offsets` empty, which is the step a struct literal makes easy to get
         * wrong: leaving a signature in place does not make the bytes a
         * precondition, it just leaves bytes nobody reads.
         */
        (CuteGirlKind::MikotoChanFile, via detect_mikoto_chan),

        /*
         * A byte signature. `at` and `read` are optional and default to
         * DEFAULT_OFFSET (0) and DEFAULT_MAX_BYTES_READ (2,048).
         */
        (CuteGirlKind::ShoujoFile, b"MagicalGirl"),
    ];

    let from_mikoto = match_types_custom(b"MikotoChan", RULES, CuteGirlKind::UnknownFallback);
    let from_magical = match_types_custom(b"MagicalGirl", RULES, CuteGirlKind::UnknownFallback);
    let from_nothing = match_types_custom(b"something else", RULES, CuteGirlKind::UnknownFallback);

    println!("{from_mikoto:?}"); /* MikotoChanFile */
    println!("{from_magical:?}"); /* ShoujoFile */
    println!("{from_nothing:?}"); /* UnknownFallback */

    // A predicate placed first wins, and that is the order the macro preserves.
    assert_eq!(from_mikoto, CuteGirlKind::MikotoChanFile);
    assert_eq!(from_magical, CuteGirlKind::ShoujoFile);
    assert_eq!(from_nothing, CuteGirlKind::UnknownFallback);
}

fn main() {
    my_custom_magic();
    my_custom_magic_with_fn();
    my_magic_detect();
    my_magic_detect_with_the_macro();
    magic_custom_macro::magic_custom_any();
    magic_custom_macro::magic_custom_all();
}
