pub struct Source {
    pub id: i32,
    pub url: String,
    pub last_checked: String,
}

pub struct NewSource<'a> {
    pub url: &'a str,
    pub last_checked: &'a str,
}

pub struct Activity {
    pub id: i32,
    pub source_id: i32,
    pub post_url: String,
    pub timestamp: String,
}

pub struct NewActivity<'a> {
    pub source_id: i32,
    pub post_url: &'a str,
    pub timestamp: &'a str,
}
