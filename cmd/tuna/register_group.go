package main

import (
	"encoding/json"
	"fmt"
	"log/slog"
	"os"
	"strings"

	"github.com/chromedp/chromedp"
	"github.com/lukasl-dev/tuna/pkg/tuna"
)

type registerGroup struct {
	Headless bool `short:"h" help:"Headless" default:"true"`

	Username string `short:"u" help:"TU Username" required:"true"`
	Password string `short:"p" help:"TU Password" required:"true"`
	TOTP     string `short:"t" help:"TOTP Url" default:""`

	Semester string `short:"s" help:"The semester to register in" required:"true"`
	Course   string `short:"c" help:"The course to register in" required:"true"`
	Group    string `short:"g" help:"The group name to register in" required:"true"`
}

func (r registerGroup) Run() error {
	ctx, cancel := chromedpContext(r.Headless)
	defer cancel()

	creds, err := credentials(r.Username, r.Password, r.TOTP)
	if err != nil {
		slog.Error("failed to build credentials", "error", err)
		return err
	}

	var groups []tuna.Group

	err = chromedp.Run(ctx, chromedp.Tasks{
		tuna.Login(creds),
		tuna.RegisterGroup(r.Semester, r.Course, r.Group),
		tuna.ListGroups(&groups, r.Semester, r.Course),
	})
	if err != nil {
		captureErrorScreenshot(ctx, "register-group")
		slog.Error("flow failed", "error", err)
		return err
	}

	selectedGroup := strings.TrimSpace(r.Group)
	slog.Info("selected group", "group", selectedGroup)

	var selected *tuna.Group
	for _, group := range groups {
		if strings.EqualFold(strings.TrimSpace(group.Name), selectedGroup) {
			g := group
			selected = &g
			break
		}
	}

	if selected == nil {
		return fmt.Errorf("selected group %q not present in listed groups", selectedGroup)
	}

	enc := json.NewEncoder(os.Stdout)
	enc.SetIndent("", "  ")
	if err = enc.Encode(selected); err != nil {
		return err
	}

	_, err = os.Stdout.WriteString("\n")
	return err
}
