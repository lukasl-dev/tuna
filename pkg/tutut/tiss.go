package tutut

import (
	"context"
	"fmt"
	"log/slog"
	"strings"
	"time"

	"github.com/chromedp/chromedp"
)

const timeLayout = "02.01.2006, 15:04"

type CourseRegistartion struct {
	Username string
	Password string
	TOTP     string

	Semester string
	CourseNr string
}

func RegisterInCourse(in CourseRegistartion) chromedp.Action {
	url := fmt.Sprintf(
		"https://tiss.tuwien.ac.at/course/educationDetails.xhtml?semester=%s&courseNr=%s",
		in.Semester,
		in.CourseNr,
	)

	return chromedp.Tasks{
		login(in.Username, in.Password, in.TOTP),
		chromedp.Navigate(url),
	}
}

type GroupRegistration struct {
	Username string
	Password string
	TOTP     string

	Candidates []string
	Semester   string
	CourseNr   string
}

func RegisterInGroup(in GroupRegistration) chromedp.Action {
	var groups []Group

	// TODO:

	return chromedp.Tasks{
		login(in.Username, in.Password, in.TOTP),
		listGroups(&groups, in.Semester, in.CourseNr),
		chromedp.ActionFunc(func(ctx context.Context) error {
			for _, group := range groups {
				slog.Debug("parsed group", "group", group)
			}
			return nil
		}),
	}
}

type Group struct {
	Name             string
	Participants     int
	MaxParticipants  int
	ApplicationBegin time.Time
}

func listGroups(dst *[]Group, semester, courseNr string) chromedp.Action {
	url := fmt.Sprintf(
		"https://tiss.tuwien.ac.at/education/course/groupList.xhtml?semester=%s&courseNr=%s",
		semester,
		courseNr,
	)

	return chromedp.ActionFunc(func(ctx context.Context) error {
		err := chromedp.Run(ctx, chromedp.Navigate(url))
		if err != nil {
			return err
		}

		err = chromedp.Run(ctx, chromedp.WaitVisible(".groupWrapper", chromedp.ByQuery))
		if err != nil {
			return err
		}

		type groupRaw struct {
			Name                string `json:"name"`
			Participants        int    `json:"participants"`
			MaxParticipants     int    `json:"maxParticipants"`
			ApplicationBeginRaw string `json:"applicationBegin"`
		}

		var rawGroups []groupRaw
		err = chromedp.Run(ctx, chromedp.EvaluateAsDevTools(`(() => {
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
})()`, &rawGroups))
		if err != nil {
			return err
		}

		for _, raw := range rawGroups {
			group := Group{
				Name:            strings.TrimSpace(raw.Name),
				Participants:    raw.Participants,
				MaxParticipants: raw.MaxParticipants,
			}

			appBeginText := strings.TrimSpace(raw.ApplicationBeginRaw)
			if appBeginText != "" {
				appBegin, parseErr := time.Parse(timeLayout, appBeginText)
				if parseErr != nil {
					return parseErr
				}
				group.ApplicationBegin = appBegin
			}

			*dst = append(*dst, group)
		}

		return nil
	})
}
