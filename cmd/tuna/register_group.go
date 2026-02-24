package main

type registerGroup struct {
	Username string `short:"u" help:"TU Username" required:"true"`
	Password string `short:"p" help:"TU Password" required:"true"`
	TOTP     string `short:"t" help:"TOTP Url" default:""`
}

func (r registerGroup) Run() error {
	return nil
}
