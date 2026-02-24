package main

import (
	"log/slog"
	"os"
	"os/signal"

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

	err = chromedp.Run(ctx, chromedp.Tasks{
		tuna.Login(creds),
		tuna.RegisterGroup(r.Semester, r.Course, r.Group),
	})
	if err != nil {
		slog.Error("flow failed", "error", err)
		return err
	}

	sig := make(chan os.Signal, 1)
	signal.Notify(sig, os.Interrupt)
	<-sig

	return nil
}
