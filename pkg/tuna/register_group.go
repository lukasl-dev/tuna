package tuna

import (
	"context"
	"fmt"
	"log/slog"
	"strings"

	"github.com/chromedp/chromedp"
)

func RegisterGroup(semester, course, group string) chromedp.Action {
	url := fmt.Sprintf(
		"https://tiss.tuwien.ac.at/education/course/groupList.xhtml?semester=%s&courseNr=%s",
		semester,
		course,
	)

	groupListPanelSelector := `#groupContentForm\:groupListPanel`
	confirmRegistrationButtonSelector := `form[id="regForm"] ul.styledCommandBox li:first-child input[type="submit"][name^="regForm:"]`
	registrationSuccessSelector := `form[id="confirmForm"] div.staticInfoMessage`

	type clickResult struct {
		Clicked         bool     `json:"clicked"`
		MatchedGroup    string   `json:"matchedGroup"`
		ButtonLabel     string   `json:"buttonLabel"`
		AvailableGroups []string `json:"availableGroups"`
		PanelText       string   `json:"panelText"`
		Error           string   `json:"error"`
	}

	var result clickResult
	registrationSuccessMessage := ""

	clickScript := fmt.Sprintf(`(() => {
  const target = %q;
  const normalize = (s) => (s || '').replace(/\s+/g, ' ').trim().toLowerCase();

  const panel = document.querySelector('#groupContentForm\\:groupListPanel');
  const panelText = panel?.textContent?.replace(/\s+/g, ' ').trim() || '';

  const wrappers = Array.from(document.querySelectorAll('.groupWrapper'));
  const availableGroups = wrappers
    .map((wrapper) => wrapper.querySelector('.groupHeaderWrapper .titleColStudent span.bold')?.textContent?.trim() || '')
    .filter((name) => name !== '');

  for (const wrapper of wrappers) {
    const name = wrapper.querySelector('.groupHeaderWrapper .titleColStudent span.bold')?.textContent?.trim() || '';
    if (normalize(name) !== normalize(target)) continue;

    const button = wrapper.querySelector('input[type="submit"][name^="groupContentForm:"]');
    if (!button) {
      return {
        clicked: false,
        matchedGroup: name,
        availableGroups,
        panelText,
        error: 'matching group has no submit button',
      };
    }

    button.click();
    return {
      clicked: true,
      matchedGroup: name,
      buttonLabel: button.value || '',
      availableGroups,
      panelText,
    };
  }

  return {
    clicked: false,
    availableGroups,
    panelText,
  };
})()`, group)

	return chromedp.Tasks{
		debug("navigating to group registration page", "url", url, "group", group),
		chromedp.Navigate(url),

		debug("waiting for group list panel"),
		waitVisible(groupListPanelSelector, chromedp.ByQuery),

		debug("clicking group registration button", "group", group),
		chromedp.EvaluateAsDevTools(clickScript, &result),

		chromedp.ActionFunc(func(context.Context) error {
			if result.Error != "" {
				return fmt.Errorf("failed to click group registration button for %q: %s", group, result.Error)
			}

			if result.Clicked {
				return nil
			}

			if len(result.AvailableGroups) == 0 {
				if result.PanelText != "" {
					return fmt.Errorf("no groups available for course %s in %s: %s", course, semester, result.PanelText)
				}
				return fmt.Errorf("no groups available for course %s in %s", course, semester)
			}

			return fmt.Errorf(
				"group %q not found, available groups: %s",
				group,
				strings.Join(result.AvailableGroups, ", "),
			)
		}),

		debug("waiting for confirmation registration button"),
		waitVisible(confirmRegistrationButtonSelector, chromedp.ByQuery),
		waitEnabled(confirmRegistrationButtonSelector, chromedp.ByQuery),

		debug("clicking confirmation registration button"),
		chromedp.Click(confirmRegistrationButtonSelector, chromedp.ByQuery),

		debug("waiting for group registration success message"),
		waitVisible(registrationSuccessSelector, chromedp.ByQuery),

		debug("reading group registration success message"),
		chromedp.Text(registrationSuccessSelector, &registrationSuccessMessage, chromedp.ByQuery),
		chromedp.ActionFunc(func(context.Context) error {
			slog.Info(
				"group registration completed",
				"semester", semester,
				"course", course,
				"group", group,
				"message", strings.TrimSpace(registrationSuccessMessage),
			)
			return nil
		}),
	}
}
