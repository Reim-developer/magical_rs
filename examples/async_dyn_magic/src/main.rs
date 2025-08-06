/*
* From version `0.3.0` and above, you can use
* the `AsyncDynMagic` feature.
* For safety reasons and you need to know what
* are you doing, you need to use Cargo's features
* flag to use that.
*
* cargo add magical_rs --features magical_async_dyn
*
* Note: Only use this when you really need asynchronous processing
* and you know what the hell you're doing. I warned you.
*/

use async_std::task;
use magical_rs::magical::async_dyn_magic::AsyncDynMagic;
use magical_rs::magical::async_dyn_magic::match_dyn_types_as;
use std::time::Duration;

/*
* Of course, to use this feature, you need an async function.
*/
async fn magic_async_detect() {
    /*
     * Create your async rule function here.
     * Do whatever you want.
     */
    let func_detect = |bytes: &[u8]| {
        let owned_bytes = bytes.to_vec();

        Box::pin(async move {
            /*
             * Here, you can do whatever the hell you want.
             * Train AI? Send request to server to analyzer bytes?
             * Open GUI, then asking user what type of file is ?
             * Send report to China ?
             *
             * Everything is valid. Limit? Your brain.
             *
             * Oh, you can even `rm -rf`, no one forbirds it.
             * No,.. i..i it wrong, forget it, don't do bad things :<
             * Live a good life and a very moe anime girl will
             * come find you and kiss you <3.
             */
            println!("Rest for 1 second");

            task::sleep(Duration::from_millis(1000)).await;
            owned_bytes.starts_with(b"Magical")
        })
    };

    let rule = AsyncDynMagic::new(func_detect, "Magical_File", 128);
    let rules = vec![rule];

    /*
     * Make sure the type you cast is correct or your
     * runtime crashes
     */
    let result = match_dyn_types_as::<&str>(b"Magical", &rules).await;

    /*
     * Using match in this case is always recommended.
     */
    match result {
        Some(r) => println!("Magical File Detect: {r}"),
        None => println!("Magical File Not Found"),
    }
}

#[async_std::main]
async fn main() {
    magic_async_detect().await;
}
