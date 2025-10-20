package module

import (
	"encoding/json"

	"cosmossdk.io/core/appmodule"
	"github.com/binary-builders/astro/simapp/x/astro/keeper"
	"github.com/binary-builders/astro/simapp/x/astro/types"
	"github.com/grpc-ecosystem/grpc-gateway/runtime"

	abci "github.com/cometbft/cometbft/abci/types"
	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/codec"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkmodule "github.com/cosmos/cosmos-sdk/types/module"
)

// AppModuleBasic implements the sdk AppModuleBasic interface for x/astro.
type AppModuleBasic struct{}

func (b AppModuleBasic) RegisterGRPCGatewayRoutes(context client.Context, mux *runtime.ServeMux) {
	// TODO implement me
	panic("implement me")
}

var _ sdkmodule.AppModuleBasic = AppModuleBasic{}

func (AppModuleBasic) Name() string { return types.ModuleName }
func (AppModuleBasic) RegisterLegacyAminoCodec(cdc *codec.LegacyAmino) {
	types.RegisterLegacyAminoCodec(cdc)
}
func (AppModuleBasic) RegisterInterfaces(reg codectypes.InterfaceRegistry) {
	types.RegisterInterfaces(reg)
}
func (AppModuleBasic) DefaultGenesis(cdc codec.JSONCodec) json.RawMessage {
	return cdc.MustMarshalJSON(types.DefaultGenesis())
}
func (AppModuleBasic) ValidateGenesis(cdc codec.JSONCodec, _ client.TxEncodingConfig, bz json.RawMessage) error {
	var gs types.GenesisState
	if err := cdc.UnmarshalJSON(bz, &gs); err != nil {
		return err
	}
	return gs.Validate()
}

// AppModule implements the sdk AppModule interface for x/astro.
type AppModule struct {
	AppModuleBasic

	keeper keeper.Keeper
}

func (am AppModule) IsOnePerModuleType() {}

func (am AppModule) IsAppModule() {}

var _ appmodule.AppModule = AppModule{}

func NewAppModule(_ codec.Codec, k keeper.Keeper) AppModule {
	return AppModule{keeper: k}
}

func (am AppModule) Name() string { return types.ModuleName }

func (am AppModule) RegisterServices(_ sdkmodule.Configurator) {}

func (am AppModule) InitGenesis(ctx sdk.Context, cdc codec.JSONCodec, data json.RawMessage) []abci.ValidatorUpdate {
	var gs types.GenesisState
	cdc.MustUnmarshalJSON(data, &gs)

	// The keeper InitGenesis uses context.Context; bridge from sdk.Context
	if err := am.keeper.InitGenesis(ctx, gs); err != nil {
		panic(err)
	}
	return []abci.ValidatorUpdate{}
}

func (am AppModule) ExportGenesis(ctx sdk.Context, cdc codec.JSONCodec) json.RawMessage {
	gs, err := am.keeper.ExportGenesis(ctx)
	if err != nil {
		panic(err)
	}
	return cdc.MustMarshalJSON(gs)
}

func (am AppModule) ConsensusVersion() uint64 { return 1 }
