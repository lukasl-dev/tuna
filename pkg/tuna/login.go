package tuna

import (
	"github.com/chromedp/chromedp"
)

type LoginCredentials struct {
	Username string
	Password string
	TOTP     string
}

func Login(creds LoginCredentials) chromedp.Action {
	const url = "https://tiss.tuwien.ac.at/admin/authentifizierung"

	t := chromedp.Tasks{
		debug("navigating to login page"),
		chromedp.Navigate(url),

		debug("waiting for login page"),
		chromedp.WaitVisible("#core\\:loginuserpass", chromedp.ByQuery),

		debug("waiting for login fields"),
		chromedp.WaitReady("username", chromedp.ByID),
		chromedp.WaitReady("password", chromedp.ByID),

		debug("typing username into login form"),
		chromedp.SendKeys("username", creds.Username, chromedp.ByID),

		debug("typing password into login form"),
		chromedp.SendKeys("password", creds.Password, chromedp.ByID),
	}

	if creds.TOTP != "" {
		t = append(t,
			debug("typing totp into login form"),
			chromedp.SendKeys("totp", creds.TOTP, chromedp.ByID),
		)
	}

	return append(t,
		debug("submitting login form"),
		chromedp.Submit("samlloginbutton", chromedp.ByID),

		debug("wait until login finishes"),
		chromedp.WaitVisible("a.toolLogout", chromedp.ByQuery),
	)
}
