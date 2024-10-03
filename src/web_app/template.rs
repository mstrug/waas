

pub const HTML_HEAD: &str = r##"
    <!DOCTYPE html>
    <html>
    <head>
        <meta name="viewport" content="width=device-width, initial-scale=1">
        <meta charset="UTF-8">
        <title>Wallet as a service</title>
        <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/bulma@1.0.2/css/bulma.min.css">
    </head>"##;

pub const HTML_BODY_NAVBAR: &str = r##"<body>
<section class="hero has-background-light is-fullheight">
  <!-- Hero head: will stick at the top -->
  <div class="hero-head">
    <nav class="navbar">
      <div class="container">
        <div class="navbar-brand">
          <a class="navbar-item">
            <p class="subtitle is-2">Wallet as a service</p>
          </a>
        </div>
        <div id="navbarMenuHeroA" class="navbar-menu">
          <div class="navbar-end">
            {menu-items}
          </div>
        </div>
      </div>
    </nav>
  </div>"##;
pub const HTML_NAVBAR_MENU_ITEM_PLACEHOLDER: &str = "{menu-items}";
pub const HTML_NAVBAR_MENU_ITEM_LOGOUT: &str = r##"<a class="navbar-item" href="/logout"> Logout </a>"##;
pub const HTML_NAVBAR_MENU_ITEM_GENERATE_KEY: &str = r##"<a class="navbar-item" href="/key/generate"> Generate Key </a>"##;
pub const HTML_NAVBAR_MENU_ITEM_DISCARD_KEY: &str = r##"<a class="navbar-item" href="/key/discard"> Discard Key </a>"##;

pub const HTML_BODY_CONTENT: &str = r##"<!-- Hero content: will be in the middle -->
  <div class="hero-body">
    <div class="container ">
    <div class="columns is-mobile is-centered">
        <div class="column is-half">
            {body-content}
        </div>
    </div>
    </div>
  </div>"##;
pub const HTML_BODY_CONTENT_PLACEHOLDER: &str = "{body-content}";
pub const HTML_BODY_CONTENT_LOGIN: &str = r##"<form action="/login" method="post">
                <div class="field">
                    <label class="label is-medium">Provide login credentials</label>
                    <div class="control">
                        <input class="input is-medium" type="text" placeholder="Username" name="username" value="user1" required/>
                    </div>
                </div>
                <div class="field">
                    <div class="control">
                        <input class="input is-medium" type="password" placeholder="Password" name="password" value="123456" required/>
                    </div>
                </div>
                <div class="field is-grouped is-grouped-centered">
                    <p class="control">
                        <button class="button is-primary" type="submit">Login</button>
                    </p>
                </div>
            </form>"##;
pub const HTML_USERNAME_PLACEHOLDER: &str = "{user}";
pub const HTML_BODY_CONTENT_NO_KEY: &str = r##"
    <div class="block"><p class="subtitle is-3">Hello {user}!</p></div>
    <div class="block">It looks like you haven't generated a key yet.</div>
    <div class="block">To do so, click on the <strong>Generate Key</strong> option in the upper right corner.</div>"##;
pub const HTML_BODY_CONTENT_SIGN_MESSAGE: &str = r##"<p>Sign message.</p>"##;

pub const HTML_BODY_FOOTER: &str = r##"
  <!-- Hero footer: will stick at the bottom -->
  <div class="hero-foot">
    <div class="content has-text-centered m-2">
    <p>
      <strong>Rust Fullstack demo</strong> by <a href="https://strug.pl">Michał Strug</a> using Poem and Bulma CSS.
    </p>
  </div>
  </div>
</section>

    </body>
    </html>"##;
