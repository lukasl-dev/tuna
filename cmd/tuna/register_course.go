package main

import (
	"context"
	"log/slog"
	"time"

	"github.com/chromedp/chromedp"
	"github.com/lukasl-dev/tuna/pkg/tuna"
	"github.com/pquerna/otp"
	"github.com/pquerna/otp/totp"
)

type registerCourse struct {
	Headless bool `short:"h" help:"Headless" default:"true"`

	Username string `short:"u" help:"TU Username" required:"true"`
	Password string `short:"p" help:"TU Password" required:"true"`
	TOTP     string `short:"t" help:"TOTP Url" default:""`

	Semester string `short:"s" help:"The semester to register for" required:"true"`
	Course   string `short:"c" help:"The course to register for" required:"true"`
}

func (r registerCourse) Run() error {
	slog.Debug("starting register-course", "headless", r.Headless, "has_totp_url", r.TOTP != "")

	allocatorCtx, cancelAllocator := chromedp.NewExecAllocator(context.TODO(),
		append(
			chromedp.DefaultExecAllocatorOptions[:],
			chromedp.Flag("headless", r.Headless),
		)...,
	)
	defer cancelAllocator()

	ctx, cancel := chromedp.NewContext(allocatorCtx)
	defer cancel()

	creds, err := r.credentials()
	if err != nil {
		slog.Error("failed to build credentials", "error", err)
		return err
	}

	reg := tuna.CourseRegistration{Semester: r.Semester, Course: r.Course}

	err = chromedp.Run(ctx, chromedp.Tasks{
		tuna.Login(creds),
		tuna.RegisterCourse(reg),
	})
	if err != nil {
		slog.Error("login flow failed", "error", err)
		return err
	}

	return nil
}

func (r registerCourse) credentials() (tuna.LoginCredentials, error) {
	var code string

	if r.TOTP != "" {
		slog.Debug("parsing TOTP URL")
		key, err := otp.NewKeyFromURL(r.TOTP)
		if err != nil {
			slog.Error("failed to parse TOTP URL", "error", err)
			return tuna.LoginCredentials{}, err
		}

		slog.Debug("generating TOTP code")
		code, err = totp.GenerateCode(key.Secret(), time.Now())
		if err != nil {
			slog.Error("failed to generate TOTP code", "error", err)
			return tuna.LoginCredentials{}, nil
		}
	}

	creds := tuna.LoginCredentials{
		Username: r.Username,
		Password: r.Password,
		TOTP:     code,
	}
	return creds, nil
}
