package main

import (
	"encoding/json"
	"log/slog"
	"os"

	"github.com/chromedp/chromedp"
	"github.com/lukasl-dev/tuna/pkg/tuna"
)

type listGroups struct {
	Headless bool `short:"h" help:"Headless" default:"true"`

	Username string `short:"u" help:"TU Username" required:"true"`
	Password string `short:"p" help:"TU Password" required:"true"`
	TOTP     string `short:"t" help:"TOTP Url" default:""`

	Semester string `short:"s" help:"The semester to get groups for" required:"true"`
	Course   string `short:"c" help:"The course to get groups for" required:"true"`
}

func (l listGroups) Run() error {
	ctx, cancel := chromedpContext(l.Headless)
	defer cancel()

	creds, err := credentials(l.Username, l.Password, l.TOTP)
	if err != nil {
		slog.Error("failed to build credentials", "error", err)
		return err
	}

	var groups []tuna.Group

	err = chromedp.Run(ctx, chromedp.Tasks{
		tuna.Login(creds),
		tuna.ListGroups(&groups, l.Semester, l.Course),
	})
	if err != nil {
		slog.Error("flow failed", "error", err)
		return err
	}

	if len(groups) == 0 {
		if _, err := os.Stdout.WriteString("[]\n"); err != nil {
			return err
		}
		return nil
	}

	b, err := json.MarshalIndent(groups, "", "  ")
	if err != nil {
		return err
	}

	if _, err := os.Stdout.Write(append(b, '\n')); err != nil {
		return err
	}

	return nil
}
