package tuna

import (
	"context"
	"fmt"
	"log/slog"
	"strings"

	"github.com/chromedp/chromedp"
)

func RegisterCourse(semester, course string) chromedp.Action {
	url := fmt.Sprintf(
		"https://tiss.tuwien.ac.at/education/course/courseRegistration.xhtml?semester=%s&courseNr=%s",
		semester,
		course,
	)

	// IMPORTANT:
	// assumes that the user is already logged in

	registrationButtonSelector := `#registrationForm\:formContentPanel input[type="submit"][name^="registrationForm:"]`
	confirmRegistrationButtonSelector := `form[id="regForm"] ul.styledCommandBox li:first-child input[type="submit"][name^="regForm:"]`
	registrationSuccessSelector := `form[id="confirmForm"] div.staticInfoMessage`
	registrationSuccessMessage := ""

	t := chromedp.Tasks{
		debug("navigating to course registration page", "url", url),
		chromedp.Navigate(url),

		debug("waiting for registration button"),
		waitVisible(registrationButtonSelector, chromedp.ByQuery),
		waitEnabled(registrationButtonSelector, chromedp.ByQuery),

		debug("clicking registration button"),
		chromedp.Click(registrationButtonSelector, chromedp.ByQuery),

		debug("waiting for confirmation registration button"),
		waitVisible(confirmRegistrationButtonSelector, chromedp.ByQuery),
		waitEnabled(confirmRegistrationButtonSelector, chromedp.ByQuery),

		debug("clicking confirmation registration button"),
		chromedp.Click(confirmRegistrationButtonSelector, chromedp.ByQuery),

		debug("waiting for registration success message"),
		waitVisible(registrationSuccessSelector, chromedp.ByQuery),

		debug("reading registration success message"),
		chromedp.Text(registrationSuccessSelector, &registrationSuccessMessage, chromedp.ByQuery),
		chromedp.ActionFunc(func(context.Context) error {
			slog.Info(
				"course registration completed",
				"semester", semester,
				"course", course,
				"message", strings.TrimSpace(registrationSuccessMessage),
			)
			return nil
		}),
	}

	return t
}
