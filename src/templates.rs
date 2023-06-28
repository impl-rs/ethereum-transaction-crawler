use maud::{html, Markup, DOCTYPE};

fn header(page_title: &str) -> Markup {
    html! {
        head {
            (DOCTYPE)
            meta charset="utf-8";
            title { (page_title) }
            style {
                "table, th, td {
                    border: 1px solid black;
                }"
            }
        }

    }
}

pub fn form() -> Markup {
    html! {
        h1 { "Insert wallet address and block number to search" }
        form action="/" {
            input type="text" name="address" placeholder="Wallet address" value="0x62c7c75b46E86FAdd27928D0F6de1df22276860e";
            input type="text" name="block" placeholder="Block number" value="17571440";
            input type="submit" value="Submit";
        }
    }
}

pub fn body(page_title: &str, content: Markup) -> Markup {
    html! {
        body {
            (header(page_title))
            body {
                (content)
            }
        }
    }
}
