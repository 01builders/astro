package cli

import (
	"crypto/rand"
	"encoding/hex"
	"encoding/json"
	"errors"
	"fmt"

	"github.com/spf13/cobra"
	blst "github.com/supranational/blst/bindings/go"

	"github.com/binary-builders/astro/simapp/x/astro/types"
	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/flags"
	"github.com/cosmos/cosmos-sdk/server"
	"github.com/cosmos/cosmos-sdk/x/genutil"
	genutiltypes "github.com/cosmos/cosmos-sdk/x/genutil/types"
)

const (
	flagJSON  = "json"
	flagForce = "force"
)

type blsGenesisOutput struct {
	Moniker          string `json:"moniker"`
	Scheme           string `json:"scheme"`
	PublicKeyHex     string `json:"public_key_hex"`
	PublicKeyFormat  string `json:"public_key_format"`
	PrivateKeyHex    string `json:"private_key_hex"`
	GenesisFilePath  string `json:"genesis_file"`
	ValidatorSetNote string `json:"validator_set"`
}

// NewGenesisBLSCmd wires a module-native genesis command that generates a
// BLS12-381 MinSig keypair and writes a single Astro validator plus network
// identity into the x/astro genesis state.
func NewGenesisBLSCmd(defaultNodeHome string) *cobra.Command {
	cmd := &cobra.Command{
		Use:   "astro-bls-validator [moniker]",
		Short: "Generate a BLS MinSig key and set x/astro genesis identity + single validator",
		Args:  cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			moniker := args[0]

			clientCtx := client.GetClientContextFromCmd(cmd)
			serverCtx := server.GetServerContextFromCmd(cmd)
			cfg := serverCtx.Config
			cfg.SetRoot(clientCtx.HomeDir)

			force, _ := cmd.Flags().GetBool(flagForce)
			jsonOut, _ := cmd.Flags().GetBool(flagJSON)

			key, err := generateMinSigKeypair()
			if err != nil {
				return err
			}

			appState, appGenesis, err := genutiltypes.GenesisStateFromGenFile(cfg.GenesisFile())
			if err != nil {
				return fmt.Errorf("failed to read genesis file: %w", err)
			}

			var astroGenesis types.GenesisState
			if len(appState[types.ModuleName]) > 0 {
				if err := clientCtx.Codec.UnmarshalJSON(appState[types.ModuleName], &astroGenesis); err != nil {
					return fmt.Errorf("failed to decode astro genesis state: %w", err)
				}
			} else {
				astroGenesis = *types.DefaultGenesis()
			}

			if len(astroGenesis.Identity) > 0 && !force {
				return fmt.Errorf("astro genesis identity already set; use --%s to overwrite", flagForce)
			}
			if len(astroGenesis.Validators) > 0 && !force {
				return fmt.Errorf("astro genesis already has validators; use --%s to replace with a single entry", flagForce)
			}

			validator := types.Validator{
				Moniker:  moniker,
				Identity: key.publicKey,
			}

			astroGenesis.Identity = key.publicKey
			astroGenesis.Validators = []*types.Validator{&validator}

			if err := astroGenesis.Validate(); err != nil {
				return fmt.Errorf("invalid astro genesis: %w", err)
			}
			if err := types.ValidateIdentity(astroGenesis.Identity); err != nil {
				return fmt.Errorf("invalid astro identity: %w", err)
			}

			appState[types.ModuleName] = clientCtx.Codec.MustMarshalJSON(&astroGenesis)

			appStateJSON, err := json.Marshal(appState)
			if err != nil {
				return fmt.Errorf("failed to marshal updated app state: %w", err)
			}

			appGenesis.AppState = appStateJSON
			if err := genutil.ExportGenesisFile(appGenesis, cfg.GenesisFile()); err != nil {
				return fmt.Errorf("failed to write genesis file: %w", err)
			}

			output := blsGenesisOutput{
				Moniker:          moniker,
				Scheme:           key.scheme,
				PublicKeyHex:     hex.EncodeToString(key.publicKey),
				PublicKeyFormat:  "compressed G2 (96 bytes) used for astro identity + validator",
				PrivateKeyHex:    hex.EncodeToString(key.privateKey),
				GenesisFilePath:  cfg.GenesisFile(),
				ValidatorSetNote: "astro genesis now contains exactly one validator with this key",
			}

			if jsonOut {
				enc := json.NewEncoder(cmd.OutOrStdout())
				enc.SetIndent("", "  ")
				return enc.Encode(output)
			}

			fmt.Fprintf(cmd.OutOrStdout(), "wrote astro genesis with 1 validator (moniker=%s)\n", moniker)
			fmt.Fprintf(cmd.OutOrStdout(), "genesis file: %s\n", output.GenesisFilePath)
			fmt.Fprintf(cmd.OutOrStdout(), "public_key (hex): %s\n", output.PublicKeyHex)
			fmt.Fprintln(cmd.OutOrStdout(), "public_key_format: compressed G2 (96 bytes) used for astro identity + validator")
			fmt.Fprintf(cmd.OutOrStdout(), "private_key (hex): %s\n", output.PrivateKeyHex)
			return nil
		},
	}

	cmd.Flags().Bool(flagJSON, false, "print the generated key info as JSON")
	cmd.Flags().Bool(flagForce, false, "overwrite any existing astro genesis identity/validators with the generated key")
	cmd.Flags().String(flags.FlagHome, defaultNodeHome, "node's home directory")
	return cmd
}

type minSigKeypair struct {
	scheme     string
	privateKey []byte
	publicKey  []byte
}

func generateMinSigKeypair() (*minSigKeypair, error) {
	var ikm [32]byte
	if _, err := rand.Read(ikm[:]); err != nil {
		return nil, fmt.Errorf("read entropy: %w", err)
	}

	sk := blst.KeyGen(ikm[:])
	if sk == nil {
		return nil, errors.New("failed to derive BLS secret key")
	}

	pk := new(blst.P2Affine).From(sk)
	if pk == nil {
		return nil, errors.New("failed to derive BLS public key")
	}

	compressed := pk.Compress()
	return &minSigKeypair{
		scheme:     "BLS12-381 MinSig (pubkey in G2, signatures in G1)",
		privateKey: sk.Serialize(),
		publicKey:  compressed,
	}, nil
}
