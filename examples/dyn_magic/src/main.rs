/*
* Here, we will try out the advanced feature of
* magical_rs: DynMagic
*
* Warning:
* You can do anything with `DynMagic`, there are no limits.
* If you abuse it, you will suffer serious consequences.
*/
use magical_rs::magical::dyn_magic::DynMagicCustom;

/*
* Here you can write a function, where you can do whatever condition
* you want as long as you make sure it a closure
* that returns a `bool` & with a parmeter `&[u8]`.
*/
fn my_detect_rule() -> impl Fn(&[u8]) -> bool {
    let require_bytes = b"MagicalGirl";

    |bytes: &[u8]| bytes.starts_with(require_bytes) && bytes.len() == require_bytes.len()
}

fn detect_custom_file(file_bytes: &'static [u8]) -> bool {
    let detect_fn = my_detect_rule();

    /*
     * Here, define your rules.
     * Matcher is the rule you just defined
     * Kind represents 'if that rule, matches' what will value be?
     * Max bytes read is the maxium number of bytes that need to be read.
     */
    let rule = DynMagicCustom::new(detect_fn, String::from("Is Mahou Shoujo Detect."), 32);

    /*
     * And, of course, you can get that value easily, but be careful.
     * Because if the type is wrong, it will be cause undefined beavior.
     */
    let kind = rule.kind_downcast_ref::<String>();

    /*
     * I recommend you use match to avoid undefined beavior.
     */
    match kind {
        Some(k) => println!("{k}"), /* Is Mahou Shoujo Detect. */
        None => println!("Kind not found."),
    }

    rule.matches(file_bytes)
}

fn main() {
    let my_bytes = b"MagicalGirl";

    assert!(detect_custom_file(my_bytes));
}
