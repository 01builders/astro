package astro_test

import (
	"encoding/json"
	"path/filepath"
	"testing"

	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/codec"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	genutil "github.com/cosmos/cosmos-sdk/x/genutil"
	genutiltypes "github.com/cosmos/cosmos-sdk/x/genutil/types"

	"github.com/binary-builders/astro/simapp/x/astro"
	"github.com/binary-builders/astro/simapp/x/astro/types"
)

func TestAddGenesisValidator(t *testing.T) {
	cdc := newTestCodec(t)
	genesisFile := writeGenesisFile(t, cdc, types.DefaultGenesis())

	val := types.Validator{Moniker: "node-1", Identity: []byte{0x01, 0x02}}
	require.NoError(t, astro.AddGenesisValidator(cdc, genesisFile, val, false))

	state := readAstroGenesis(t, cdc, genesisFile)
	require.Len(t, state.Validators, 1)
	require.Equal(t, "node-1", state.Validators[0].Moniker)
	require.Equal(t, []byte{0x01, 0x02}, state.Validators[0].Identity)
}

func TestAddGenesisValidatorDuplicate(t *testing.T) {
	cdc := newTestCodec(t)
	genesisFile := writeGenesisFile(t, cdc, types.DefaultGenesis())

	val := types.Validator{Moniker: "duplicate", Identity: []byte{0x01}}
	require.NoError(t, astro.AddGenesisValidator(cdc, genesisFile, val, false))

	err := astro.AddGenesisValidator(cdc, genesisFile, val, false)
	require.Error(t, err)
	require.Contains(t, err.Error(), "already exists")
}

func TestAddGenesisValidatorReplace(t *testing.T) {
	cdc := newTestCodec(t)
	genesisFile := writeGenesisFile(t, cdc, types.DefaultGenesis())

	val := types.Validator{Moniker: "node", Identity: []byte{0x01}}
	require.NoError(t, astro.AddGenesisValidator(cdc, genesisFile, val, false))

	replacement := types.Validator{Moniker: "node", Identity: []byte{0xAA}}
	require.NoError(t, astro.AddGenesisValidator(cdc, genesisFile, replacement, true))

	state := readAstroGenesis(t, cdc, genesisFile)
	require.Len(t, state.Validators, 1)
	require.Equal(t, []byte{0xAA}, state.Validators[0].Identity)
}

func newTestCodec(t *testing.T) codec.Codec {
	t.Helper()

	reg := codectypes.NewInterfaceRegistry()
	types.RegisterInterfaces(reg)

	return codec.NewProtoCodec(reg)
}

func writeGenesisFile(t *testing.T, cdc codec.Codec, gs *types.GenesisState) string {
	t.Helper()

	if gs == nil {
		gs = types.DefaultGenesis()
	}
	if gs.Params == nil {
		p := types.DefaultParams()
		gs.Params = &p
	}

	bz, err := cdc.MarshalJSON(gs)
	require.NoError(t, err)

	appState := map[string]json.RawMessage{
		types.ModuleName: bz,
	}
	appStateBz, err := json.Marshal(appState)
	require.NoError(t, err)

	appGenesis := &genutiltypes.AppGenesis{
		AppName:    "astro",
		AppVersion: "v0.1.0",
		ChainID:    "astro-local-1",
		AppState:   appStateBz,
		Consensus:  &genutiltypes.ConsensusGenesis{},
	}

	genFile := filepath.Join(t.TempDir(), "genesis.json")
	require.NoError(t, genutil.ExportGenesisFile(appGenesis, genFile))

	return genFile
}

func readAstroGenesis(t *testing.T, cdc codec.Codec, genesisFile string) *types.GenesisState {
	t.Helper()

	appState, _, err := genutiltypes.GenesisStateFromGenFile(genesisFile)
	require.NoError(t, err)

	state := types.DefaultGenesis()
	require.NoError(t, cdc.UnmarshalJSON(appState[types.ModuleName], state))

	return state
}
