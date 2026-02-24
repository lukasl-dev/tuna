package tuna

import (
	"context"
	"fmt"
	"strings"
	"time"

	"github.com/chromedp/chromedp"
)

const groupTimeLayout = "02.01.2006, 15:04"

type Group struct {
	Name             string
	Participants     uint
	MaxParticipants  uint
	ApplicationBegin time.Time
}

func ListGroups(dst *[]Group, semester, course string) chromedp.Action {
	url := fmt.Sprintf(
		"https://tiss.tuwien.ac.at/education/course/groupList.xhtml?semester=%s&courseNr=%s",
		semester,
		course,
	)

	type groupRaw struct {
		Name                string `json:"name"`
		Participants        int    `json:"participants"`
		MaxParticipants     int    `json:"maxParticipants"`
		ApplicationBeginRaw string `json:"applicationBegin"`
	}
	var rawGroups []groupRaw
	groupListPanelSelector := `#groupContentForm\:groupListPanel`

	// IMPORTANT:
	// assumes that the user is already logged in

	return chromedp.Tasks{
		debug("navigating to group list page", "url", url),
		chromedp.Navigate(url),

		debug("waiting for group list panel"),
		chromedp.WaitVisible(groupListPanelSelector, chromedp.ByQuery),

		debug("extracting raw groups"),
		chromedp.EvaluateAsDevTools(`(() => {
  const groups = Array.from(document.querySelectorAll('.groupWrapper'));
  return groups.map(group => {
    const name = group.querySelector('.groupHeaderWrapper .titleColStudent span.bold')?.textContent?.trim() || '';
    const lis = Array.from(group.querySelectorAll('li'));
    const participantsLi = lis.find(li => li.querySelector('label')?.textContent?.trim() === 'Participants');
    const appBeginLi = lis.find(li => li.querySelector('label')?.textContent?.trim() === 'Application begin');
    const participantsText = participantsLi?.textContent || '';
    const appBeginText = appBeginLi?.querySelector('span')?.textContent?.trim() || '';
    const nums = (participantsText.match(/\d+/g) || []).map(n => Number(n));

    return {
      name,
      participants: nums[0] || 0,
      maxParticipants: nums[1] || 0,
      applicationBegin: appBeginText,
    };
  });
})()`, &rawGroups),

		debug("parsing groups"),
		chromedp.ActionFunc(func(context.Context) error {
			if dst == nil {
				return fmt.Errorf("groups destination must not be nil")
			}

			groups := make([]Group, 0, len(rawGroups))
			for _, raw := range rawGroups {
				group := Group{
					Name: strings.TrimSpace(raw.Name),
				}

				if raw.Participants > 0 {
					group.Participants = uint(raw.Participants)
				}
				if raw.MaxParticipants > 0 {
					group.MaxParticipants = uint(raw.MaxParticipants)
				}

				appBeginText := strings.TrimSpace(raw.ApplicationBeginRaw)
				if appBeginText != "" {
					appBegin, err := time.Parse(groupTimeLayout, appBeginText)
					if err != nil {
						return err
					}
					group.ApplicationBegin = appBegin
				}

				groups = append(groups, group)
			}

			*dst = append(*dst, groups...)
			return nil
		}),
	}
}
