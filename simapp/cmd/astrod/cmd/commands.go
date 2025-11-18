package cmd

import (
	"errors"
	"fmt"
	"io"

	cmtcfg "github.com/cometbft/cometbft/config"
	dbm "github.com/cosmos/cosmos-db"
	"github.com/spf13/cobra"
	"github.com/spf13/viper"

	"cosmossdk.io/log"
	confixcmd "cosmossdk.io/tools/confix/cmd"

	"github.com/binary-builders/astro/simapp"
	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/debug"
	"github.com/cosmos/cosmos-sdk/client/keys"
	"github.com/cosmos/cosmos-sdk/client/pruning"
	"github.com/cosmos/cosmos-sdk/client/rpc"
	"github.com/cosmos/cosmos-sdk/client/snapshot"
	"github.com/cosmos/cosmos-sdk/server"
	serverconfig "github.com/cosmos/cosmos-sdk/server/config"
	servertypes "github.com/cosmos/cosmos-sdk/server/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/module"
	authcmd "github.com/cosmos/cosmos-sdk/x/auth/client/cli"
	genutilcli "github.com/cosmos/cosmos-sdk/x/genutil/client/cli"
)

const (
	withCometFlag   = "with-comet"
	transportFlag   = "transport"
	grpcTransport   = "grpc"
	startCmdSummary = "Run the Astro ABCI gRPC server"
)

func initRootCmd(rootCmd *cobra.Command, txConfig client.TxConfig, basicManager module.BasicManager) {
	cfg := sdk.GetConfig()
	cfg.Seal()

	rootCmd.AddCommand(
		genutilcli.InitCmd(basicManager, simapp.DefaultNodeHome),
		debug.Cmd(),
		confixcmd.ConfigCommand(),
		pruning.Cmd(newApp, simapp.DefaultNodeHome),
		snapshot.Cmd(newApp),
	)

	server.AddCommandsWithStartCmdOptions(rootCmd, simapp.DefaultNodeHome, newApp, appExport, server.StartCmdOptions{
		AddFlags: configureGrpcStartCmd,
	})

	rootCmd.AddCommand(
		server.StatusCommand(),
		genesisCommand(txConfig, basicManager),
		queryCommand(),
		txCommand(),
		keys.Commands(),
	)
}

func configureGrpcStartCmd(cmd *cobra.Command) {
	if err := forceFlagDefault(cmd, withCometFlag, "false"); err != nil {
		panic(fmt.Errorf("failed to update --%s default: %w", withCometFlag, err))
	}
	if err := forceFlagDefault(cmd, transportFlag, grpcTransport); err != nil {
		panic(fmt.Errorf("failed to update --%s default: %w", transportFlag, err))
	}

	if flag := cmd.Flags().Lookup(withCometFlag); flag != nil {
		flag.Hidden = true
		flag.Usage = "Astro always runs out-of-process; CometBFT embedding is disabled."
	}

	if flag := cmd.Flags().Lookup(transportFlag); flag != nil {
		flag.Hidden = true
		flag.Usage = "Transport is fixed to gRPC so the Commonware consensus daemon can connect."
	}

	cmd.Short = startCmdSummary
	cmd.Long = startCmdSummary + ". This command only serves the ABCI application over gRPC; it never embeds CometBFT."
	cmd.PreRunE = chainPreRuns(cmd.PreRunE, ensureGrpcOnlyStart)
}

func ensureGrpcOnlyStart(cmd *cobra.Command, _ []string) error {
	if cmd.Flags().Changed(withCometFlag) {
		withCmt, err := cmd.Flags().GetBool(withCometFlag)
		if err != nil {
			return err
		}
		if withCmt {
			return fmt.Errorf("CometBFT in-process mode is disabled; remove --%s", withCometFlag)
		}
	}

	transport, err := cmd.Flags().GetString(transportFlag)
	if err != nil {
		return err
	}
	if transport != grpcTransport {
		return fmt.Errorf("ABCI transport must be %q", grpcTransport)
	}

	serverCtx := server.GetServerContextFromCmd(cmd)
	serverCtx.Viper.Set(withCometFlag, false)
	serverCtx.Viper.Set(transportFlag, grpcTransport)

	return nil
}

func chainPreRuns(existing, extra func(cmd *cobra.Command, args []string) error) func(cmd *cobra.Command, args []string) error {
	switch {
	case existing == nil:
		return extra
	case extra == nil:
		return existing
	default:
		return func(cmd *cobra.Command, args []string) error {
			if err := existing(cmd, args); err != nil {
				return err
			}
			return extra(cmd, args)
		}
	}
}

func forceFlagDefault(cmd *cobra.Command, name, value string) error {
	flag := cmd.Flags().Lookup(name)
	if flag == nil {
		return fmt.Errorf("flag %s not found", name)
	}

	if err := flag.Value.Set(value); err != nil {
		return err
	}
	flag.DefValue = value
	flag.Changed = false

	return nil
}

// initCometBFTConfig mirrors the reference configuration overrides so we can
// adjust them later without touching the CLI surfaces.
func initCometBFTConfig() *cmtcfg.Config {
	cfg := cmtcfg.DefaultConfig()
	return cfg
}

// initAppConfig allows us to tweak default app.toml values.
func initAppConfig() (string, interface{}) {
	type CustomConfig struct {
		CustomField string `mapstructure:"custom-field"`
	}

	type CustomAppConfig struct {
		serverconfig.Config `mapstructure:",squash"`
		Custom              CustomConfig `mapstructure:"custom"`
	}

	srvCfg := serverconfig.DefaultConfig()
	srvCfg.MinGasPrices = "0stake"

	customAppConfig := CustomAppConfig{
		Config: *srvCfg,
		Custom: CustomConfig{
			CustomField: "astro",
		},
	}

	customAppTemplate := serverconfig.DefaultConfigTemplate + `
[custom]
custom-field = "{{ .Custom.CustomField }}"`

	return customAppTemplate, customAppConfig
}

func genesisCommand(txConfig client.TxConfig, basicManager module.BasicManager, cmds ...*cobra.Command) *cobra.Command {
	cmd := genutilcli.Commands(txConfig, basicManager, simapp.DefaultNodeHome)
	for _, subCmd := range cmds {
		cmd.AddCommand(subCmd)
	}
	return cmd
}

func queryCommand() *cobra.Command {
	cmd := &cobra.Command{
		Use:                        "query",
		Aliases:                    []string{"q"},
		Short:                      "Querying subcommands",
		DisableFlagParsing:         false,
		SuggestionsMinimumDistance: 2,
		RunE:                       client.ValidateCmd,
	}

	cmd.AddCommand(
		rpc.WaitTxCmd(),
		server.QueryBlockCmd(),
		authcmd.QueryTxsByEventsCmd(),
		server.QueryBlocksCmd(),
		authcmd.QueryTxCmd(),
		server.QueryBlockResultsCmd(),
	)

	return cmd
}

func txCommand() *cobra.Command {
	cmd := &cobra.Command{
		Use:                        "tx",
		Short:                      "Transactions subcommands",
		DisableFlagParsing:         false,
		SuggestionsMinimumDistance: 2,
		RunE:                       client.ValidateCmd,
	}

	cmd.AddCommand(
		authcmd.GetSignCommand(),
		authcmd.GetSignBatchCommand(),
		authcmd.GetMultiSignCommand(),
		authcmd.GetMultiSignBatchCmd(),
		authcmd.GetValidateSignaturesCommand(),
		authcmd.GetBroadcastCommand(),
		authcmd.GetEncodeCommand(),
		authcmd.GetDecodeCommand(),
		authcmd.GetSimulateCmd(),
	)

	return cmd
}

func newApp(logger log.Logger, db dbm.DB, traceStore io.Writer, appOpts servertypes.AppOptions) servertypes.Application {
	baseappOptions := server.DefaultBaseappOptions(appOpts)
	return simapp.NewSimApp(
		logger,
		db,
		traceStore,
		true,
		appOpts,
		baseappOptions...,
	)
}

func appExport(
	logger log.Logger,
	db dbm.DB,
	traceStore io.Writer,
	height int64,
	forZeroHeight bool,
	jailAllowedAddrs []string,
	appOpts servertypes.AppOptions,
	modulesToExport []string,
) (servertypes.ExportedApp, error) {
	viperAppOpts, ok := appOpts.(*viper.Viper)
	if !ok {
		return servertypes.ExportedApp{}, errors.New("appOpts is not viper.Viper")
	}

	viperAppOpts.Set(server.FlagInvCheckPeriod, 1)
	appOpts = viperAppOpts

	var application *simapp.SimApp
	if height != -1 {
		application = simapp.NewSimApp(logger, db, traceStore, false, appOpts)

		if err := application.LoadHeight(height); err != nil {
			return servertypes.ExportedApp{}, err
		}
	} else {
		application = simapp.NewSimApp(logger, db, traceStore, true, appOpts)
	}

	return application.ExportAppStateAndValidators(forZeroHeight, jailAllowedAddrs, modulesToExport)
}
