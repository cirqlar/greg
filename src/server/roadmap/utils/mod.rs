use itertools::Itertools;
use log::error;

pub(super) fn clean_description(mut s: String) -> String {
    s = s.replace("<span data-preserve-white-space></span>", "\n");
    s = s.replace("<p>", "\n");
    s = s.replace("</p>", "");
    s = s.replace("\\(", "(");
    s = s.replace("\\)", ")");
    s = s.replace("\\/", "/");
    s = s.replace("\\+", "+");
    s = s.replace("**", "");

    let mut dot_loop = 0;
    // Remove . that isn't \.
    loop {
        if dot_loop > 1000 {
            error!("Failed dot clean on now string {s}");
            break;
        }

        let Some(index) = s
            .bytes()
            .tuple_windows()
            .enumerate()
            .find_map(|(index, (one, us))| (one == b' ' && us == b'.').then_some(index))
        else {
            break;
        };
        s.replace_range(index..(index + 2), "\n\t.");

        dot_loop += 1;
    }

    let mut dash_loop = 0;
    // Remove - that isn't \-
    loop {
        if dash_loop > 1000 {
            error!("Failed dash clean on now string {s}");
            break;
        }

        let Some(index) = s
            .bytes()
            .tuple_windows()
            .enumerate()
            .find_map(|(index, (one, us))| (one == b' ' && us == b'-').then_some(index))
        else {
            break;
        };
        s.replace_range(index..(index + 2), "\n\t-");

        dash_loop += 1;
    }

    // unescape
    s = s.replace("\\.", ".");
    s = s.replace("\\-", "-");

    s
}
