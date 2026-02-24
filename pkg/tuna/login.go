package tuna

import (
	"context"
	"fmt"
	"strings"
	"time"

	"github.com/chromedp/chromedp"
)

type LoginCredentials struct {
	Username string
	Password string
	TOTP     string
}

func Login(creds LoginCredentials) chromedp.Action {
	const url = "https://tiss.tuwien.ac.at/admin/authentifizierung"
	const loginInputDelay = 150 * time.Millisecond
	type loginState struct {
		LoggedIn       bool   `json:"loggedIn"`
		LoginErrorText string `json:"loginErrorText"`
	}

	const loginStateScript = `(() => {
  const logout = document.querySelector('a.toolLogout');
  const loginError = document.querySelector('.message-box.error h3')?.textContent?.trim() || '';

  return {
    loggedIn: Boolean(logout),
    loginErrorText: loginError,
  };
})()`

	t := chromedp.Tasks{
		debug("navigating to login page"),
		chromedp.Navigate(url),

		debug("waiting for login page"),
		waitVisible("#core\\:loginuserpass", chromedp.ByQuery),

		debug("waiting for login fields"),
		waitReady("username", chromedp.ByID),
		waitReady("password", chromedp.ByID),

		debug("typing username into login form"),
		chromedp.SendKeys("username", creds.Username, chromedp.ByID),
		chromedp.Sleep(loginInputDelay),

		debug("typing password into login form"),
		chromedp.SendKeys("password", creds.Password, chromedp.ByID),
		chromedp.Sleep(loginInputDelay),
	}

	if creds.TOTP != "" {
		t = append(t,
			debug("typing totp into login form"),
			chromedp.SendKeys("totp", creds.TOTP, chromedp.ByID),
			chromedp.Sleep(loginInputDelay),
		)
	}

	return append(t,
		debug("submitting login form"),
		chromedp.Submit("samlloginbutton", chromedp.ByID),

		debug("waiting for login result"),
		withTimeout(chromedp.ActionFunc(func(ctx context.Context) error {
			for {
				var state loginState
				if err := chromedp.Run(ctx, chromedp.EvaluateAsDevTools(loginStateScript, &state)); err != nil {
					return err
				}

				if state.LoggedIn {
					return nil
				}

				if state.LoginErrorText != "" {
					return fmt.Errorf("login failed: %s", strings.TrimSpace(state.LoginErrorText))
				}

				select {
				case <-ctx.Done():
					return ctx.Err()
				case <-time.After(100 * time.Millisecond):
				}
			}
		})),
	)
}
