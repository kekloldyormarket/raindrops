pub mod utils;

use {
    crate::utils::{
        assert_derivation, assert_initialized, assert_owned_by, create_or_allocate_account_raw,
        get_mask_and_index_for_seq, spl_token_burn, spl_token_mint_to, spl_token_transfer,
        TokenBurnParams, TokenTransferParams,
    },
    anchor_lang::{
        prelude::*,
        solana_program::{
            program::{invoke, invoke_signed},
            program_option::COption,
            program_pack::Pack,
            system_instruction, system_program,
        },
        AnchorDeserialize, AnchorSerialize,
    },
    anchor_spl::token::{Mint, TokenAccount},
    metaplex_token_metadata::instruction::{
        create_master_edition, create_metadata_accounts,
        mint_new_edition_from_master_edition_via_token, update_metadata_accounts,
    },
    spl_token::{
        instruction::{initialize_account2, mint_to},
        state::Account,
    },
};
anchor_lang::declare_id!("p1exdMJcjVao65QdewkaZRUnU6VPSXhus9n2GzWfh98");
pub const PLAYER_ID: Pubkey =
    Pubkey::from_str("p1exdMJcjVao65QdewkaZRUnU6VPSXhus9n2GzWfh98").unwrap();

#[program]
pub mod item {
    use super::*;
}

// [COMMON REMAINING ACCOUNTS]
// Most actions require certain remainingAccounts based on their permissioned setup
// if you see common remaining accounts label, use the following as your rubric:
// If update/usage permissiveness is token holder can update:
// token_account [readable]
// token_holder [signer]
// If update/usage permissiveness is class holder can update
// class token_account [readable]
// class token_holder [signer]
// class [readable]
// class mint [readable]
// If update/usage permissiveness is update authority can update
// metadata_update_authority [signer]
// metadata [readable]
// If update permissiveness is anybody can update, nothing further is required.

#[derive(Accounts)]
#[instruction( item_class_bump: u8, class_index: u64, space: usize)]
pub struct CreateItemClass<'info> {
    // parent determines who can create this (if present) so need to add all classes and check who is the signer...
    // perhaps do this via optional additional accounts to save space.
    #[account(init, seeds=[PREFIX.as_bytes(), item_mint.key().as_ref(), &class_index.to_le_bytes()], bump=item_class_bump, space=space, payer=payer, constraint=space >= MIN_ITEM_CLASS_SIZE)]
    item_class: Account<'info, ItemClass>,
    item_mint: Account<'info, Mint>,
    metadata: UncheckedAccount<'info>,
    edition: UncheckedAccount<'info>,
    // is the parent item class (if there is one.)
    parent: UncheckedAccount<'info>,
    payer: Signer<'info>,
    system_program: Program<'info, System>,
    rent: Sysvar<'info, Rent>,
    // If parent is unset, need to provide:
    // metadata_update_authority [signer]
    // If parent is set, and update permissiveness is token holder can update:
    // parent token_account [readable]
    // parent token_holder [signer]
    // parent mint [readable]
    // If parent is set, and update permissiveness is class holder can update
    // parent's class token_account [readable]
    // parent's class token_holder [signer]
    // parent's class [readable]
    // parent's class's mint [readable]
    // If parent is set, and update permissiveness is namespace holder can update
    // namespace token_account [readable]
    // namespace token_holder [signer]
    // namespace [readable]
    // If parent is set and update permissiveness is update authority can update
    // parent's metadata_update_authority [signer]
    // parent's metadata [readable]
    // parent's mint [readable]
    // If parent is set and update permissiveness is anybody can update, nothing further is required.
}

#[derive(Accounts)]
#[instruction(craft_bump: u8, class_index: u64, index: u64)]
pub struct CreateItemEscrow<'info> {
    // parent determines who can create this (if present) so need to add all classes and check who is the signer...
    // perhaps do this via optional additional accounts to save space.
    #[account(mut, seeds=[PREFIX.as_bytes(), item_mint.key().as_ref(), &class_index.to_le_bytes()], bump=item_class.bump)]
    item_class: Account<'info, ItemClass>,
    item_class_mint: Account<'info, Mint>,
    new_item_mint: Account<'info, Mint>,
    new_item_metadata: UncheckedAccount<'info>,
    new_item_edition: UncheckedAccount<'info>,
    #[account(init, seeds=[PREFIX.as_bytes(), item_mint.key().as_ref(), payer.key().as_ref(), new_item_mint.key().as_ref(), &index.to_le_bytes()], bump=craft_bump, space=8+1+8+1, payer=payer)]
    item_escrow: Account<'info, ItemEscrow>,
    #[account(constraint=new_item_token.mint == new_item_mint.key() && new_item_token.amount > 0)]
    new_item_token: Account<'info, TokenAccount>,
    // may be required signer if builder must be holder in item class is true
    new_item_token_holder: UncheckedAccount<'info>,
    payer: Signer<'info>,
    system_program: Program<'info, System>,
    rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(token_bump: u8, class_index: u64, index: u64)]
pub struct AddCraftItemToEscrow<'info> {
    #[account(mut, seeds=[PREFIX.as_bytes(), item_mint.key().as_ref(), &class_index.to_le_bytes()], bump=item_class.bump)]
    item_class: Account<'info, ItemClass>,
    item_class_mint: Account<'info, Mint>,
    new_item_mint: Account<'info, Mint>,
    // payer is in seed so that draining funds can only be done by original payer
    #[account(mut, seeds=[PREFIX.as_bytes(), item_mint.key().as_ref(), originator.key().as_ref(), new_item_mint.key().as_ref(),&index.to_le_bytes()], bump=item_escrow.bump)]
    item_escrow: Account<'info, ItemEscrow>,
    #[account(constraint=new_item_token.mint == new_item_mint.key() && new_item_token.amount > 0)]
    new_item_token: Account<'info, TokenAccount>,
    // may be required signer if builder must be holder in item class is true
    new_item_token_holder: UncheckedAccount<'info>,
    // cant be stolen to a different craft item token account due to seed by token key
    #[account(init, seeds=[PREFIX.as_bytes(), item_mint.key().as_ref(), new_item_mint.key().as_ref(),payer.key().as_ref(), ccraft_item_token_account.key().as_ref(),&index.to_le_bytes(),craft_item_token_account.mint.as_ref()], bump=token_bump,token::mint = craft_item_token_account.mint, token::authority = item_class.key(), payer=payer)]
    craft_item_token_account_escrow: Account<'info, TokenAccount>,
    #[account(mut)]
    craft_item_token_account: Account<'info, TokenAccount>,
    craft_item_transfer_authority: Signer<'info>,
    payer: Signer<'info>,
    originator: UncheckedAccount<'info>,
    system_program: Program<'info, System>,
    rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(token_bump: u8, class_index: u64, index: u64)]
pub struct RemoveCraftItemFromEscrow<'info> {
    #[account(mut, seeds=[PREFIX.as_bytes(), item_mint.key().as_ref(), &class_index.to_le_bytes()], bump=item_class.bump)]
    item_class: Account<'info, ItemClass>,
    item_class_mint: Account<'info, Mint>,
    new_item_mint: Account<'info, Mint>,
    #[account(mut, seeds=[PREFIX.as_bytes(), item_mint.key().as_ref(), originator.key().as_ref(), new_item_mint.key().as_ref(), &index.to_le_bytes()], bump=item_escrow.bump)]
    item_escrow: Account<'info, ItemEscrow>,
    #[account(constraint=new_item_token.mint == new_item_mint.key() && new_item_token.amount > 0)]
    new_item_token: Account<'info, TokenAccount>,
    // may be required signer if builder must be holder in item class is true
    new_item_token_holder: UncheckedAccount<'info>,
    // cant be stolen to a different craft item token account due to seed by token key
    #[account(mut, seeds=[PREFIX.as_bytes(), item_mint.key().as_ref(), new_item_mint.key().as_ref(),receiver.key().as_ref(), craft_item_token_account.key().as_ref(), &index.to_le_bytes(), craft_item_token_account.mint.as_ref()], bump=token_bump)]
    craft_item_token_account_escrow: Account<'info, TokenAccount>,
    #[account(mut)]
    craft_item_token_account: Account<'info, TokenAccount>,
    // if craft item is burned and mint supply -> 0, lamports are returned from this account as well to kill the item off completely in the gamespace
    #[account(mut, seeds=[PREFIX.as_bytes(), craft_item_mint.key().as_ref()], bump=craft_item.bump)]
    craft_item: Account<'info, Item>,
    #[account(constraint=craft_item.parent == craft_item_class.key())]
    craft_item_class: Account<'info, ItemClass>,
    craft_item_mint: Account<'info, Mint>,
    // account funds will be drained here from craft_item_token_account_escrow
    receiver: Signer<'info>,
    originator: UncheckedAccount<'info>,
    system_program: Program<'info, System>,
    rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(class_index: u64, index: u64)]
pub struct DeactivateItemEscrow<'info> {
    #[account(mut, seeds=[PREFIX.as_bytes(), item_mint.key().as_ref(), &class_index.to_le_bytes()], bump=item_class.bump)]
    item_class: Account<'info, ItemClass>,
    item_class_mint: Account<'info, Mint>,
    new_item_mint: Account<'info, Mint>,
    #[account(mut, seeds=[PREFIX.as_bytes(), item_mint.key().as_ref(), originator.key().as_ref(), new_item_mint.key().as_ref(), &index.to_le_bytes()], bump=item_escrow.bump)]
    item_escrow: Account<'info, ItemEscrow>,
    #[account(constraint=new_item_token.mint == new_item_mint.key() && new_item_token.amount > 0)]
    new_item_token: Account<'info, TokenAccount>,
    // may be required signer if builder must be holder in item class is true
    new_item_token_holder: UncheckedAccount<'info>,
    originator: Signer<'info>,
}
#[derive(Accounts)]
#[instruction(class_index: u64, index: u64)]
pub struct DrainItemEscrow<'info> {
    #[account(mut, seeds=[PREFIX.as_bytes(), item_mint.key().as_ref(), &class_index.to_le_bytes()], bump=item_class.bump)]
    item_class: Account<'info, ItemClass>,
    item_class_mint: Account<'info, Mint>,
    new_item_mint: Account<'info, Mint>,
    #[account(mut, seeds=[PREFIX.as_bytes(), item_mint.key().as_ref(), originator.key().as_ref(), new_item_mint.key().as_ref(), &index.to_le_bytes()], bump=item_escrow.bump)]
    item_escrow: Account<'info, ItemEscrow>,
    originator: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(new_item_bump: u8, class_index: u64, index: u64, space: usize)]
pub struct CompleteItemEscrow<'info> {
    // parent determines who can create this (if present) so need to add all classes and check who is the signer...
    // perhaps do this via optional additional accounts to save space.
    #[account(mut, seeds=[PREFIX.as_bytes(), item_mint.key().as_ref(), &class_index.to_le_bytes()], bump=item_class.bump)]
    item_class: Account<'info, ItemClass>,
    item_class_mint: Account<'info, Mint>,
    #[account(init, seeds=[PREFIX.as_bytes(), new_item_mint.key().as_ref(), &index.to_le_bytes()], bump=new_item_bump, payer=payer, space=space, constraint= space >= MIN_ITEM_SIZE)]
    new_item: Account<'info, ItemClass>,
    new_item_mint: Account<'info, Mint>,
    #[account(mut, seeds=[PREFIX.as_bytes(), item_mint.key().as_ref(), originator.key().as_ref(), new_item_mint.key().as_ref(), &index.to_le_bytes()], bump=item_escrow.bump)]
    item_escrow: Account<'info, ItemEscrow>,
    #[account(constraint=new_item_token.mint == new_item_mint.key() && new_item_token.amount > 0)]
    new_item_token: Account<'info, TokenAccount>,
    // may be required signer if builder must be holder in item class is true
    new_item_token_holder: UncheckedAccount<'info>,
    payer: Signer<'info>,
    originator: UncheckedAccount<'info>,
    system_program: Program<'info, System>,
    rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(class_index: u64)]
pub struct UpdateItemClass<'info> {
    #[account(mut, seeds=[PREFIX.as_bytes(), item_mint.key().as_ref(), &class_index.to_le_bytes()], bump=item_class.bump)]
    item_class: Account<'info, ItemClass>,
    item_mint: Account<'info, Mint>,
    // See the [COMMON REMAINING ACCOUNTS] ctrl f for this
}

#[derive(Accounts)]
#[instruction(class_index: u64)]
pub struct DrainItemClass<'info> {
    #[account(mut, seeds=[PREFIX.as_bytes(), item_mint.key().as_ref(), &class_index.to_le_bytes()], bump=item_class.bump)]
    item_class: Account<'info, ItemClass>,
    item_mint: Account<'info, Mint>,
    receiver: Signer<'info>,
    // See the [COMMON REMAINING ACCOUNTS] ctrl f for this
}

#[derive(Accounts)]
#[instruction(index: u64)]
pub struct DrainItem<'info> {
    #[account(mut, seeds=[PREFIX.as_bytes(), item_mint.key().as_ref(), &index.to_le_bytes()], bump=item.bump)]
    item: Account<'info, Item>,
    item_mint: Account<'info, Mint>,
    receiver: Signer<'info>,
    // See the [COMMON REMAINING ACCOUNTS] ctrl f for this
}

#[derive(Accounts)]
#[instruction(item_activation_bump: u8, index: u64)]
pub struct BeginItemActivation<'info> {
    #[account( seeds=[PREFIX.as_bytes(), item_mint.key().as_ref(), &index.to_le_bytes()], bump=item.bump)]
    item: Account<'info, Item>,
    #[account(init, seeds=[PREFIX.as_bytes(), item_mint.key().as_ref(), &index.to_le_bytes(), MARKER.as_bytes()], bump=item_activation_bump, space=9, payer=payer)]
    item_activation_marker: UncheckedAccount<'info>,
    item_mint: Account<'info, Mint>,
    // payer required here as extra key to guarantee some paying entity for anchor
    // however this signer should match one of the signers in COMMON REMAINING ACCOUNTS
    payer: Signer<'info>,
    #[account(constraint = player_program.key() == PLAYER_ID)]
    player_program: UncheckedAccount<'info>,
    // See the [COMMON REMAINING ACCOUNTS] ctrl f for this
}

#[derive(Accounts)]
#[instruction(item_activation_bump: u8, index: u64)]
pub struct EndItemActivation<'info> {
    #[account(mut, seeds=[PREFIX.as_bytes(), item_mint.key().as_ref(), &index.to_le_bytes()], bump=item.bump)]
    item: Account<'info, Item>,
    // funds from this will be drained to the signer in common remaining accounts for safety
    #[account(mut, seeds=[PREFIX.as_bytes(), item_mint.key().as_ref(), &index.to_le_bytes(), MARKER.as_bytes()], bump=item_activation_marker.data.borrow_mut()[0])]
    item_activation_marker: UncheckedAccount<'info>,
    item_mint: Account<'info, Mint>,
    // See the [COMMON REMAINING ACCOUNTS] ctrl f for this
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct Callback(pub Pubkey, pub u64);

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub enum ItemUsage {
    Wearable {
        body_part: Vec<String>,
        category: Vec<String>,
        limit_per_part: Option<u64>,
        wearable_callback: Option<Callback>,
        basic_item_effects: Option<Vec<BasicItemEffect>>,
        usage_permissiveness: Vec<UsagePermissiveness>,
    },
    Consumable {
        category: Vec<String>,
        uses: u64,
        // If none, is assumed to be 1 (to save space)
        max_players_per_use: Option<u64>,
        item_usage_type: ItemUsageType,
        consumption_callback: Option<Callback>,
        basic_item_effects: Option<Vec<BasicItemEffect>>,
        usage_permissiveness: Vec<UsagePermissiveness>,
    },
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub enum UsagePermissiveness {
    Holder,
    ClassHolder,
    UpdateAuthority,
    Anybody,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub enum ItemUsageState {
    Wearable {
        inherited: InheritanceState,
        item_usage_type: ItemUsageTypeState, //  ITEM_USAGE_TYPE_STATE_SIZE
        basic_item_effect_states: Option<Vec<BasicItemEffectState>>, // BASIC_ITEM_EFFECT_STATE_SIZE
    },
    Consumable {
        inherited: InheritanceState,
        uses_remaining: u64,                                  // 8
        item_usage_type: ItemUsageTypeState,                  //  ITEM_USAGE_TYPE_SIZE
        basic_item_effect: Option<Vec<BasicItemEffectState>>, // BASIC_ITEM_EFFECT_SIZE
    },
}

pub const ITEM_USAGE_TYPE_SIZE: usize = 9;
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub enum ItemUsageType {
    Cooldown { duration: i64 },
    Exhaustion,
    Destruction,
    Infinite,
}

pub const ITEM_USAGE_TYPE_STATE_SIZE: usize = 9;
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub enum ItemUsageTypeState {
    Cooldown { activated_at: i64 },
    Exhaustion,
    Destruction,
    Infinite,
}

pub const BASIC_ITEM_EFFECT_STATE_SIZE: usize = 9 + 1;
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct BasicItemEffectState {
    activated_at: Option<i64>,
    specific_state: BasicItemEffectSpecificState,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub enum BasicItemEffectSpecificState {
    Increment,
    Decrement,
    IncrementPercent,
    DecrementPercent,
    IncrementPercentFromBase,
    DecrementPercentFromBase,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub enum BasicItemEffectType {
    Increment,
    Decrement,
    IncrementPercent,
    DecrementPercent,
    IncrementPercentFromBase,
    DecrementPercentFromBase,
}

pub const BASIC_ITEM_EFFECT_SIZE: usize = 8 + 25 + 33 + 9 + 9 + 9 + 50;
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct BasicItemEffect {
    amount: u64,
    stat: String,
    item_effect_type: BasicItemEffectType,
    active_duration: Option<i64>,
    staking_amount_scaler: Option<u64>,
    staking_duration_scaler: Option<u64>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub enum ComponentCondition {
    Consumed,
    Presence,
    Absence,
}
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct Component {
    mint: Pubkey,
    amount: u64,
    // if we cant count this component if its incooldown
    non_cooldown_required: bool,
    condition: ComponentCondition,
    inherited: InheritanceState,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub enum UpdatePermissiveness {
    TokenHolderCanUpdate { inherited: InheritanceState },
    ClassHolderCanUpdate { inherited: InheritanceState },
    UpdateAuthorityCanUpdate { inherited: InheritanceState },
    AnybodyCanUpdate { inherited: InheritanceState },
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub enum ChildUpdatePropagationPermissiveness {
    Class { overridable: bool },
    Usages { overridable: bool },
    Components { overridable: bool },
    UpdatePermissiveness { overridable: bool },
    ChildUpdatePropagationPermissiveness { overridable: bool },
    ChildrenMustBeEditionsPermissiveness { overridable: bool },
    BuilderMustBeHolderPermissiveness { overridable: bool },
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub enum InheritanceState {
    NotInherited,
    Inherited,
    Overriden,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct DefaultItemCategory {
    category: String,
    inherited: InheritanceState,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct NamespaceAndIndex {
    namespace: Pubkey,
    indexed: bool,
}
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct ArtifactNamespaceSetting {
    namespaces: Vec<NamespaceAndIndex>,
}

pub const MIN_ITEM_CLASS_SIZE: usize = 8 + // key
1 + // mint
1 + // metadata
1 + // edition
4 + // number of namespaces
1 + // children must be editions
4 + // number of default update permissivenesses
2 + // minimum 1 default update
4+// number of child update propagations
1 + // parent
4 + // number of usages
4 +  // number of components
3 + // roots
1; //bump

#[account]
pub struct ItemClass {
    namespaces: ArtifactNamespaceSetting,
    mint: Option<Pubkey>,
    metadata: Option<Pubkey>,
    /// If not present, only Destruction/Infinite consumption types are allowed,
    /// And no cooldowns because we can't easily track a cooldown
    /// on a mint with more than 1 coin.
    edition: Option<Pubkey>,
    parent: Option<Pubkey>,
    bump: u8,
    children_must_be_editions: bool,
    builder_must_be_holder: bool,
    default_category: DefaultItemCategory,
    default_update_permissiveness: Vec<UpdatePermissiveness>,
    child_update_propagation_permissiveness: Vec<ChildUpdatePropagationPermissiveness>,
    // The roots are merkle roots, used to keep things cheap on chain (optional)
    usage_root: Option<[u8; 32]>,
    // Used to seed the root for new items
    usage_state_root: Option<[u8; 32]>,
    component_root: Option<[u8; 32]>,
    // Note that both usages and components are mutually exclusive with usage_root and component_root - if those are set, these are considered
    // cached values, and root is source of truth. Up to you to keep them up to date.
    usages: Vec<ItemUsage>,
    components: Vec<Component>,
}

#[account]
pub struct ItemEscrow {
    namespaces: ArtifactNamespaceSetting,
    bump: u8,
    deactivated: bool,
    step: u64,
}

// can make this super cheap
pub const MIN_ITEM_SIZE: usize = 8 + // key
1 + // mint
1 + // metadata
32 + // parent
1 + //indexed
2 + // authority level
1 + // edition
4 + // number of item usages
4 + // number of namespaces
4 + // number of update permissivenesses;
1 + // root
1; //bump

#[account]
pub struct Item {
    namespaces: ArtifactNamespaceSetting,
    parent: Pubkey,
    mint: Option<Pubkey>,
    metadata: Option<Pubkey>,
    /// If not present, only Destruction/Infinite consumption types are allowed,
    /// And no cooldowns because we can't easily track a cooldown
    /// on a mint with more than 1 coin.
    edition: Option<Pubkey>,
    bump: u8,
    update_permissiveness: Option<Vec<UpdatePermissiveness>>,
    usage_state_root: Option<[u8; 32]>,
    // if state root is set, usage states is considered a cache, not source of truth
    usage_states: Vec<ItemUsageState>,
}

#[error]
pub enum ErrorCode {
    #[msg("Account does not have correct owner!")]
    IncorrectOwner,
    #[msg("Account is not initialized!")]
    Uninitialized,
    #[msg("Mint Mismatch!")]
    MintMismatch,
    #[msg("Token transfer failed")]
    TokenTransferFailed,
    #[msg("Numerical overflow error")]
    NumericalOverflowError,
    #[msg("Token mint to failed")]
    TokenMintToFailed,
    #[msg("TokenBurnFailed")]
    TokenBurnFailed,
    #[msg("Derived key is invalid")]
    DerivedKeyInvalid,
}
