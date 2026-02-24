package main

import (
	"os"

	"github.com/alecthomas/kong"
)

func main() {
	var cmd root
	parser := kong.Must(&cmd,
		kong.Name("tuna"),
		kong.Description("Automate your life at TU Vienna."),
		kong.UsageOnError(),
		kong.ConfigureHelp(kong.HelpOptions{FlagsLast: true}),
	)

	ctx, err := parser.Parse(os.Args[1:])
	parser.FatalIfErrorf(err)

	parser.FatalIfErrorf(cmd.configure())
	parser.FatalIfErrorf(cmd.waitPostpone())

	parser.FatalIfErrorf(cmd.runWithRetries(func() error { return ctx.Run() }))
}
