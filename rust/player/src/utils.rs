use {
    crate::{
        AddOrRemoveItemValidationArgs, BasicStat, BasicStatState, BasicStatTemplate, BasicStatType,
        BodyPart, ChildUpdatePropagationPermissivenessType, EquippedItem, ErrorCode, ItemUsageInfo,
        Player, PlayerClass, PlayerClassData, StatDiff, StatDiffType,
    },
    anchor_lang::{
        error,
        prelude::{
            msg, Account, AccountInfo, AccountMeta, Pubkey, Rent, Result, SolanaSysvar,
            UncheckedAccount,
        },
        require,
        solana_program::{
            instruction::Instruction,
            program::{invoke, invoke_signed},
            program_pack::{IsInitialized, Pack},
            system_instruction,
        },
        AnchorSerialize, Key, ToAccountInfo,
    },
    anchor_spl::token::{Mint, TokenAccount},
    raindrops_item::{
        utils::{
            assert_keys_equal, propagate_parent, propagate_parent_array, PropagateParentArgs,
            PropagateParentArrayArgs,
        },
        BasicItemEffect, BasicItemEffectType, Item, ItemClass, ItemClassData, ItemUsage,
        ItemUsageType, PermissivenessType,
    },
    std::convert::TryInto,
};

pub fn update_player_class_with_inherited_information(
    player: &mut Account<PlayerClass>,
    player_class_data: &mut PlayerClassData,
    parent_item: &Account<PlayerClass>,
    parent_item_data: &PlayerClassData,
) {
    match &parent_item_data
        .settings
        .child_update_propagation_permissiveness
    {
        Some(cupp) => {
            for update_perm in cupp {
                match update_perm.child_update_propagation_permissiveness_type {
                    ChildUpdatePropagationPermissivenessType::StakingPermissiveness => {
                        player_class_data.settings.staking_permissiveness = propagate_parent_array(PropagateParentArrayArgs {
                            parent_items: &parent_item_data.settings.staking_permissiveness,
                            child_items: &player_class_data.settings.staking_permissiveness,
                            overridable: update_perm.overridable,
                        });

                        player_class_data.settings.unstaking_permissiveness = propagate_parent_array(PropagateParentArrayArgs {
                            parent_items: &parent_item_data.settings.unstaking_permissiveness,
                            child_items: &player_class_data.settings.unstaking_permissiveness,
                            overridable: update_perm.overridable,
                        });
                    }
                    ChildUpdatePropagationPermissivenessType::UpdatePermissiveness => {
                        player_class_data.settings.update_permissiveness =
                            propagate_parent_array(PropagateParentArrayArgs {
                                parent_items: &parent_item_data.settings.update_permissiveness,
                                child_items: &player_class_data.settings.update_permissiveness,
                                overridable: update_perm.overridable,
                            });
                    }
                    ChildUpdatePropagationPermissivenessType::BuildPermissiveness => {
                        player_class_data.settings.build_permissiveness =
                            propagate_parent_array(PropagateParentArrayArgs {
                                parent_items: &parent_item_data.settings.build_permissiveness,
                                child_items: &player_class_data.settings.build_permissiveness,
                                overridable: update_perm.overridable,
                            });
                    }
                    ChildUpdatePropagationPermissivenessType::ChildUpdatePropagationPermissiveness => {
                        player_class_data.settings.child_update_propagation_permissiveness =
                            propagate_parent_array(PropagateParentArrayArgs {
                                parent_items: &parent_item_data.settings.child_update_propagation_permissiveness,
                                child_items: &player_class_data.settings.child_update_propagation_permissiveness,
                                overridable: update_perm.overridable,
                            });
                    }
                    ChildUpdatePropagationPermissivenessType::ChildrenMustBeEditionsPermissiveness => {
                        player_class_data.settings.children_must_be_editions = propagate_parent(PropagateParentArgs {
                            parent: &parent_item_data.settings.children_must_be_editions,
                            child: &player_class_data.settings.children_must_be_editions,
                            overridable: update_perm.overridable,
                        });
                    }
                    ChildUpdatePropagationPermissivenessType::BuilderMustBeHolderPermissiveness => {
                        player_class_data.settings.builder_must_be_holder = propagate_parent(PropagateParentArgs {
                            parent: &parent_item_data.settings.builder_must_be_holder,
                            child: &player_class_data.settings.builder_must_be_holder,
                            overridable: update_perm.overridable,
                        });
                    },
                    ChildUpdatePropagationPermissivenessType::Namespaces => {
                        player.namespaces = propagate_parent_array(PropagateParentArrayArgs {
                            parent_items: &parent_item.namespaces,
                            child_items: &player.namespaces,
                            overridable: update_perm.overridable,
                        });
                    },
                    ChildUpdatePropagationPermissivenessType::InstanceUpdatePermissiveness => {
                        player_class_data.settings.instance_update_permissiveness =
                            propagate_parent_array(PropagateParentArrayArgs {
                                parent_items: &parent_item_data.settings.instance_update_permissiveness,
                                child_items: &player_class_data.settings.instance_update_permissiveness,
                                overridable: update_perm.overridable,
                            });
                    },
                    ChildUpdatePropagationPermissivenessType::EquippingItemsPermissiveness => {
                        player_class_data.settings.equip_item_permissiveness =
                            propagate_parent_array(PropagateParentArrayArgs {
                                parent_items: &parent_item_data.settings.equip_item_permissiveness,
                                child_items: &player_class_data.settings.equip_item_permissiveness,
                                overridable: update_perm.overridable,
                            });
                        player_class_data.settings.unequip_item_permissiveness =
                        propagate_parent_array(PropagateParentArrayArgs {
                            parent_items: &parent_item_data.settings.unequip_item_permissiveness,
                            child_items: &player_class_data.settings.unequip_item_permissiveness,
                            overridable: update_perm.overridable,
                        });
                    },
                    ChildUpdatePropagationPermissivenessType::AddingItemsPermissiveness =>  {
                        player_class_data.settings.add_item_permissiveness =
                            propagate_parent_array(PropagateParentArrayArgs {
                                parent_items: &parent_item_data.settings.add_item_permissiveness,
                                child_items: &player_class_data.settings.add_item_permissiveness,
                                overridable: update_perm.overridable,
                            });
                        player_class_data.settings.remove_item_permissiveness =
                        propagate_parent_array(PropagateParentArrayArgs {
                            parent_items: &parent_item_data.settings.remove_item_permissiveness,
                            child_items: &player_class_data.settings.remove_item_permissiveness,
                            overridable: update_perm.overridable,
                        });
                    },
                    ChildUpdatePropagationPermissivenessType::BasicStatTemplates => {
                        player_class_data.config.basic_stats = propagate_parent_array(PropagateParentArrayArgs {
                            parent_items: &parent_item_data.config.basic_stats,
                            child_items: &player_class_data.config.basic_stats,
                            overridable: update_perm.overridable,
                        });
                    },
                    ChildUpdatePropagationPermissivenessType::DefaultCategory => {
                        player_class_data.settings.default_category = propagate_parent(PropagateParentArgs {
                            parent: &parent_item_data.settings.default_category,
                            child: &player_class_data.settings.default_category,
                            overridable: update_perm.overridable,
                        });
                    },
                    ChildUpdatePropagationPermissivenessType::StatsUri => {
                        player_class_data.config.starting_stats_uri = propagate_parent(PropagateParentArgs {
                            parent: &parent_item_data.config.starting_stats_uri,
                            child: &player_class_data.config.starting_stats_uri,
                            overridable: update_perm.overridable,
                        });
                    },
                    ChildUpdatePropagationPermissivenessType::BodyParts => {
                        player_class_data.config.body_parts = propagate_parent_array(PropagateParentArrayArgs {
                            parent_items: &parent_item_data.config.body_parts,
                            child_items: &player_class_data.config.body_parts,
                            overridable: update_perm.overridable,
                        });
                    },
                }
            }
        }
        None => {
            // do nothing
        }
    }
}

pub fn assert_builder_must_be_holder_check(
    player_class_data: &PlayerClassData,
    new_player_token_holder: &UncheckedAccount,
) -> Result<()> {
    if let Some(b) = &player_class_data.settings.builder_must_be_holder {
        if b.boolean {
            require!(new_player_token_holder.is_signer, MustBeHolderToBuild)
        }
    }

    Ok(())
}

pub struct RunItemValidationArgs<'a, 'b, 'c, 'info> {
    pub player_class: &'b Account<'info, PlayerClass>,
    pub item_class: &'b Account<'info, ItemClass>,
    pub item: &'b Account<'info, Item>,
    pub item_account: &'b Account<'info, TokenAccount>,
    pub player_item_account: &'b Account<'info, TokenAccount>,
    pub player: &'b Account<'info, Player>,
    pub item_mint: &'b Account<'info, Mint>,
    pub validation_program: &'c UncheckedAccount<'info>,
    pub player_mint: &'a Pubkey,
    pub item_permissiveness_to_use: Option<PermissivenessType>,
    pub amount: u64,
    pub add: bool,
}

pub fn run_item_validation<'a, 'b, 'c, 'info>(
    args: RunItemValidationArgs<'a, 'b, 'c, 'info>,
) -> Result<()> {
    let RunItemValidationArgs {
        player_class,
        item_class,
        item,
        item_account,
        player_item_account,
        player,
        item_mint,
        validation_program,
        player_mint,
        item_permissiveness_to_use,
        amount,
        add,
    } = args;

    if let Some(validation) = &player_class.data.config.add_to_pack_validation {
        let item_class_info = item_class.to_account_info();
        let item_info = item.to_account_info();
        let item_account_info = item_account.to_account_info();
        let player_item_account_info = player_item_account.to_account_info();
        let player_info = player.to_account_info();
        let player_class_info = player_class.to_account_info();
        let item_mint_info = item_mint.to_account_info();
        let accounts = vec![
            item_info,
            item_class_info,
            player_info,
            player_class_info,
            item_account_info,
            player_item_account_info,
            item_mint_info,
            validation_program.to_account_info(),
        ];
        assert_keys_equal(validation_program.key(), validation.key)?;

        let keys = vec![
            AccountMeta::new_readonly(item.key(), false),
            AccountMeta::new_readonly(item_class.key(), false),
            AccountMeta::new_readonly(player.key(), false),
            AccountMeta::new_readonly(player_class.key(), false),
            AccountMeta::new_readonly(item_account.key(), false),
            AccountMeta::new_readonly(player_item_account.key(), false),
            AccountMeta::new_readonly(item_mint.key(), false),
        ];

        let name = if add {
            "add_item_validation"
        } else {
            "remove_item_validation"
        };
        invoke(
            &Instruction {
                program_id: validation.key,
                accounts: keys,
                data: AnchorSerialize::try_to_vec(&AddOrRemoveItemValidationArgs {
                    instruction: raindrops_item::utils::sighash("global", name),
                    extra_identifier: validation.code,
                    player_mint: *player_mint,
                    item_permissiveness_to_use: item_permissiveness_to_use.clone(),
                    amount,
                })?,
            },
            &accounts,
        )?;
    }

    Ok(())
}

pub fn propagate_player_class_data_fields_to_player_data(
    player: &mut Account<Player>,
    player_class: &Account<PlayerClass>,
) {
    player.namespaces = player_class.namespaces.clone();

    if player.data.stats_uri.is_none() {
        player.data.stats_uri = player_class.data.config.starting_stats_uri.clone()
    }

    if player.data.category.is_none() {
        player.data.category = player_class.data.settings.default_category.clone()
    }

    if let Some(player_stats) = &player_class.data.config.basic_stats {
        let mut states_length = 0;
        if let Some(states) = &player.data.basic_stats {
            states_length = states.len();
        }

        let mut existing_values: Vec<&BasicStat> =
            vec![
                &BasicStat {
                    index: 0,
                    state: BasicStatState::Null,
                };
                std::cmp::max(player_stats.len(), states_length)
            ];

        if let Some(states) = &player.data.basic_stats {
            for i in 0..states.len() {
                existing_values[states[i].index as usize] = &states[i];
            }
        }

        let mut new_values: Vec<BasicStat> = vec![
            BasicStat {
                index: 0,
                state: BasicStatState::Null,
            };
            std::cmp::max(player_stats.len(), states_length)
        ];

        for i in 0..player_stats.len() {
            let existing = existing_values[player_stats[i].index as usize];
            new_values[player_stats[i].index as usize] = BasicStat {
                index: player_stats[i].index,
                state: BasicStatState::Null,
            };
            let bs = &mut new_values[player_stats[i].index as usize];

            match &player_stats[i].stat_type {
                BasicStatType::Enum { starting, values } => match existing.state {
                    BasicStatState::Enum { current } => {
                        let mut found = false;
                        for val in values {
                            if val.1 == current {
                                found = true;
                            }
                        }
                        bs.state = BasicStatState::Enum {
                            current: if found { current } else { *starting },
                        };
                    }
                    _ => bs.state = BasicStatState::Enum { current: *starting },
                },
                BasicStatType::Integer { .. } => match existing.state {
                    BasicStatState::Integer { current, .. } => {
                        rebalance_basic_stat(RebalanceBasicStatArgs {
                            basic_stat: bs,
                            basic_stat_template: &player_stats[i],
                            current_change: exi,
                            ci_change: 0,
                            new_calculated_divisor: 1,
                            new_calculated_numerator: 1,
                        })?
                    }
                    _ => return Err(ErrorCode::Unreachable.into()),
                },
                BasicStatType::Bool { starting, .. } => match existing.state {
                    BasicStatState::Bool { current, .. } => {
                        bs.state = BasicStatState::Bool { current }
                    }
                    _ => bs.state = BasicStatState::Bool { current: *starting },
                },
                BasicStatType::String { starting } => match &existing.state {
                    BasicStatState::String { current, .. } => {
                        bs.state = BasicStatState::String {
                            current: current.clone(),
                        }
                    }
                    _ => {
                        bs.state = BasicStatState::String {
                            current: starting.clone(),
                        }
                    }
                },
            };
        }

        player.data.basic_stats = Some(new_values);
    }
}

pub struct VerifyItemUsageAppropriateForBodyPartArgs<'a> {
    pub used_body_part: &'a BodyPart,
    pub item_usage_index: u16,
    pub item_usage_info: Option<ItemUsageInfo>,
    pub item_class_data: ItemClassData,
    pub equipping: bool,
    pub total_equipped_for_this_item: u64,
    pub total_equipped_for_this_body_part_for_this_item: u64,
    pub total_equipped_for_this_body_part: u64,
    pub equipped_items: &'a Vec<EquippedItem>,
}

pub fn verify_item_usage_appropriate_for_body_part(
    args: VerifyItemUsageAppropriateForBodyPartArgs,
) -> Result<ItemUsage> {
    let VerifyItemUsageAppropriateForBodyPartArgs {
        used_body_part,
        item_usage_index,
        item_usage_info,
        item_class_data,
        equipping,
        total_equipped_for_this_item,
        total_equipped_for_this_body_part_for_this_item,
        total_equipped_for_this_body_part,
        equipped_items,
    } = args;

    if let Some(tis) = used_body_part.total_item_spots {
        if total_equipped_for_this_body_part > tis && equipping {
            return Err(ErrorCode::BodyPartContainsTooMany.into());
        }
    }

    let usage_to_check = if let Some(ui) = item_usage_info {
        if let Some(root) = item_class_data.config.usage_root {
            let ItemUsageInfo {
                item_usage_proof,
                item_usage,
            } = ui;
            let chief_node = anchor_lang::solana_program::keccak::hashv(&[
                &[0x00],
                &AnchorSerialize::try_to_vec(&item_usage)?,
            ]);
            require!(
                raindrops_item::utils::verify(&item_usage_proof, &root.root, chief_node.0),
                InvalidProof
            );
            item_usage
        } else {
            return Err(ErrorCode::UsageRootNotPresent.into());
        }
    } else if let Some(usages) = &item_class_data.config.usages {
        if usages.is_empty() {
            return Err(error!(ErrorCode::CannotEquipItemWithoutUsageOrMerkle));
        } else {
            let mut found = false;
            let mut item_usage = &usages[0];
            for u in usages {
                if u.index == item_usage_index {
                    item_usage = u;
                    found = true;
                    break;
                }
            }

            if !found {
                return Err(ErrorCode::FoundNoMatchingUsage.into());
            }
            item_usage.clone()
        }
    } else {
        return Err(ErrorCode::ItemContainsNoUsages.into());
    };

    if usage_to_check.do_not_pair_with_self && equipping && total_equipped_for_this_item > 0 {
        return Err(ErrorCode::ItemCannotBePairedWithSelf.into());
    }

    if let Some(dnp_list) = &usage_to_check.dnp {
        if equipping {
            for dnp_item in dnp_list {
                for equipped_item in equipped_items {
                    if equipped_item.item == dnp_item.key {
                        return Err(ErrorCode::ItemCannotBeEquippedWithDNPEntry.into());
                    }
                }
            }
        }
    }

    match &usage_to_check.item_class_type {
        raindrops_item::ItemClassType::Wearable {
            body_part,
            limit_per_part,
        } => {
            let mut found = false;
            for part in body_part {
                if &used_body_part.body_part == part {
                    found = true;
                    if let Some(lpp) = limit_per_part {
                        if total_equipped_for_this_body_part_for_this_item > *lpp && equipping {
                            return Err(ErrorCode::BodyPartContainsTooManyOfThisType.into());
                        }
                    }
                    break;
                }
            }
            if !found {
                return Err(ErrorCode::BodyPartNotEligible.into());
            }
        }
        raindrops_item::ItemClassType::Consumable { .. } => {
            return Err(ErrorCode::CannotEquipConsumable.into())
        }
    }

    return Ok(usage_to_check);
}

pub struct RunToggleEquipItemValidationArgs<'a, 'b, 'c, 'info> {
    pub item_class: &'b Account<'info, ItemClass>,
    pub item: &'b Account<'info, Item>,
    pub player_item_account: &'b Account<'info, TokenAccount>,
    pub validation_program: &'c UncheckedAccount<'info>,
    pub permissiveness_to_use: Option<PermissivenessType>,
    pub amount: u64,
    pub item_usage: ItemUsage,
    pub item_index: u64,
    pub item_class_index: u64,
    pub usage_index: u16,
    pub item_class_mint: &'a Pubkey,
}

pub fn run_toggle_equip_item_validation<'a, 'b, 'c, 'info>(
    args: RunToggleEquipItemValidationArgs<'a, 'b, 'c, 'info>,
) -> Result<()> {
    let RunToggleEquipItemValidationArgs {
        item_class,
        item,
        player_item_account,
        validation_program,
        permissiveness_to_use,
        amount,
        item_usage,
        item_index,
        item_class_index,
        usage_index,
        item_class_mint,
    } = args;

    if let Some(validation) = item_usage.validation {
        let item_class_info = item_class.to_account_info();
        let item_info = item.to_account_info();
        let item_account_info = player_item_account.to_account_info();
        let accounts = vec![
            item_class_info,
            item_info,
            item_account_info,
            validation_program.to_account_info(),
        ];
        assert_keys_equal(validation_program.key(), validation.key)?;

        let keys = vec![
            AccountMeta::new_readonly(item_class.key(), false),
            AccountMeta::new_readonly(item.key(), false),
            AccountMeta::new_readonly(player_item_account.key(), false),
        ];

        invoke(
            &Instruction {
                program_id: validation.key,
                accounts: keys,
                data: AnchorSerialize::try_to_vec(&raindrops_item::ValidationArgs {
                    instruction: raindrops_item::utils::sighash("global", "item_validation"),
                    extra_identifier: validation.code,
                    usage_permissiveness_to_use: permissiveness_to_use.clone(),
                    index: item_index,
                    amount,
                    usage_info: None,
                    usage_index,
                    class_index: item_class_index,
                    item_class_mint: *item_class_mint,
                })?,
            },
            &accounts,
        )?;
    }

    Ok(())
}

pub struct BuildNewEquippedItemsAndProvideCountsArgs<'b, 'info> {
    pub player: &'b Account<'info, Player>,
    pub item: &'b Account<'info, Item>,
    pub body_part_index: u16,
    pub amount: u64,
    pub equipping: bool,
}

pub struct BuildNewEquippedItemsReturn {
    pub total_equipped_for_this_item: u64,
    pub total_equipped_for_this_body_part_for_this_item: u64,
    pub total_equipped_for_this_body_part: u64,
    pub new_eq_items: Vec<EquippedItem>,
}

pub fn build_new_equipped_items_and_provide_counts<'b, 'info>(
    args: BuildNewEquippedItemsAndProvideCountsArgs,
) -> Result<BuildNewEquippedItemsReturn> {
    let BuildNewEquippedItemsAndProvideCountsArgs {
        player,
        item,
        body_part_index,
        amount,
        equipping,
    } = args;
    let mut new_eq_items = vec![];
    let mut moving_amount = amount;
    let mut total_equipped_for_this_item: u64 = 0;
    let mut total_equipped_for_this_body_part_for_this_item: u64 = 0;
    let mut total_equipped_for_this_body_part: u64 = 0;
    for ei in &player.equipped_items {
        if ei.item == item.key() {
            total_equipped_for_this_item = total_equipped_for_this_item
                .checked_add(ei.amount)
                .ok_or(ErrorCode::NumericalOverflowError)?;
        }

        if ei.index == body_part_index {
            total_equipped_for_this_body_part = total_equipped_for_this_body_part
                .checked_add(ei.amount)
                .ok_or(ErrorCode::NumericalOverflowError)?;

            if ei.item == item.key() {
                let new_item = EquippedItem {
                    index: ei.index,
                    item: ei.item,
                    amount: if equipping {
                        moving_amount = 0;
                        ei.amount
                            .checked_add(moving_amount)
                            .ok_or(ErrorCode::NumericalOverflowError)?
                    } else {
                        if ei.amount < moving_amount {
                            moving_amount = moving_amount
                                .checked_sub(ei.amount)
                                .ok_or(ErrorCode::NumericalOverflowError)?;
                            0
                        } else {
                            moving_amount = 0;
                            ei.amount
                                .checked_sub(moving_amount)
                                .ok_or(ErrorCode::NumericalOverflowError)?
                        }
                    },
                };
                total_equipped_for_this_body_part_for_this_item =
                    total_equipped_for_this_body_part_for_this_item
                        .checked_add(new_item.amount)
                        .ok_or(ErrorCode::NumericalOverflowError)?;
                if new_item.amount > 0 {
                    new_eq_items.push(new_item);
                }
            } else {
                new_eq_items.push(EquippedItem {
                    index: ei.index,
                    item: ei.item,
                    amount: ei.amount,
                })
            }
        } else {
            new_eq_items.push(EquippedItem {
                index: ei.index,
                item: ei.item,
                amount: ei.amount,
            })
        }
    }

    if moving_amount > 0 {
        if equipping {
            new_eq_items.push(EquippedItem {
                index: body_part_index,
                item: item.key(),
                amount: moving_amount,
            });
            total_equipped_for_this_body_part_for_this_item =
                total_equipped_for_this_body_part_for_this_item
                    .checked_add(moving_amount)
                    .ok_or(ErrorCode::NumericalOverflowError)?;
            total_equipped_for_this_body_part = total_equipped_for_this_body_part
                .checked_add(moving_amount)
                .ok_or(ErrorCode::NumericalOverflowError)?;
            total_equipped_for_this_item = total_equipped_for_this_item
                .checked_add(moving_amount)
                .ok_or(ErrorCode::NumericalOverflowError)?;
        } else {
            return Err(ErrorCode::CannotUnequipThisMuch.into());
        }
    }

    Ok(BuildNewEquippedItemsReturn {
        new_eq_items,
        total_equipped_for_this_body_part,
        total_equipped_for_this_item,
        total_equipped_for_this_body_part_for_this_item,
    })
}

pub fn find_used_body_part_from_index(
    player_class: &Account<PlayerClass>,
    body_part_index: u16,
) -> Result<BodyPart> {
    if let Some(bp) = &player_class.data.config.body_parts {
        if bp.is_empty() {
            return Err(ErrorCode::NoBodyPartsToEquip.into());
        } else {
            let mut body_part = &bp[0];
            let mut found = false;
            for b in bp {
                if b.index == body_part_index {
                    body_part = b;
                    found = true;
                    break;
                }
            }

            if !found {
                return Err(ErrorCode::UnableToFindBodyPartByIndex.into());
            }

            return Ok(body_part.clone());
        }
    };
    return Err(ErrorCode::NoBodyPartsToEquip.into());
}

pub struct ToggleItemToBasicStatsArgs<'b, 'c, 'info> {
    player: &'b mut Account<'info, Player>,
    player_class: &'b Account<'info, PlayerClass>,
    item: &'b Account<'info, Item>,
    item_usage: &'c ItemUsage,
    amount_change: u64,
    adding: bool,
    stat_diff_type: StatDiffType,
}
pub fn toggle_item_to_basic_stats<'b, 'c, 'info>(
    args: ToggleItemToBasicStatsArgs<'b, 'c, 'info>,
) -> Result<()> {
    let ToggleItemToBasicStatsArgs {
        player,
        player_class,
        item,
        item_usage,
        adding,
        amount_change,
        stat_diff_type,
    } = args;
    // for an item without active duration, is permanent increase
    // for an equipment, no active duration, is temproary until removal
    // for item with active duration, is by definition an NFT, so wont be more than one
    // for equipment, can be SFT, but then nothing can be staked on it, and if it isnt SFT, it can be staked
    // so staking issues only an issue when one of ones being used, which means if you are adding or removing
    // you can treat as having no staking adjusters because is an SFT (and thus each contributes equally)
    // or you can treat as having only one so staking adjuster will be 100% added or 100% wiped out.
    // therefore, this works.

    if let Some(bies) = &item_usage.basic_item_effects {
        if let Some(bsts) = &player_class.data.config.basic_stats {
            if let Some(bss) = &mut player.data.basic_stats {
                for bst in bsts {
                    // guaranteed to be at this index due to way player is created or updated

                    let bs = &mut bss[bst.index as usize];
                    for bie in bies {
                        if bie.stat == bst.name {
                            let modded_amount = get_modded_amount_given_tokens_staked_on_item(
                                GetModdedAmountGivenTokensStakedOnItemArgs {
                                    amount: amount_change,
                                    item,
                                    bie,
                                    adding,
                                },
                            )?;

                            if bie.active_duration.is_none()
                                && stat_diff_type == StatDiffType::Consumable
                            {
                                if adding {
                                    rebalance_stat_permanently(RebalanceStatPermanentlyArgs {
                                        bie,
                                        bs,
                                        bst,
                                        modded_amount,
                                    })?;
                                } // there is no removing a consumable with no active duration, it is permanent...
                            } else {
                                rebalance_stat_temporarily(RebalanceStatTemporarilyArgs {
                                    bie,
                                    bs,
                                    bst,
                                    modded_amount,
                                    adding,
                                })?;
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

pub struct RebalanceStatPermanentlyArgs<'a> {
    pub bie: &'a BasicItemEffect,
    pub bs: &'a mut BasicStat,
    pub bst: &'a BasicStatTemplate,
    pub modded_amount: i64,
}

pub fn rebalance_stat_permanently(args: RebalanceStatPermanentlyArgs) -> Result<()> {
    let RebalanceStatPermanentlyArgs {
        bie,
        bs,
        bst,
        modded_amount,
    } = args;
    match bie.item_effect_type {
        BasicItemEffectType::Increment | BasicItemEffectType::Decrement => {
            rebalance_basic_stat(RebalanceBasicStatArgs {
                basic_stat: bs,
                basic_stat_template: bst,
                current_change: modded_amount,
                ci_change: 0,
                new_calculated_divisor: 1,
                new_calculated_numerator: 1,
            })?
        }
        BasicItemEffectType::IncrementPercent | BasicItemEffectType::DecrementPercent => {
            match bs.state {
                BasicStatState::Integer {
                    current,

                    calculated,
                    ..
                } => rebalance_basic_stat(RebalanceBasicStatArgs {
                    basic_stat: bs,
                    basic_stat_template: bst,
                    current_change: current
                        .checked_add(
                            calculated
                                .checked_mul(
                                    100i64
                                        .checked_add(modded_amount)
                                        .ok_or(ErrorCode::NumericalOverflowError)?,
                                )
                                .ok_or(ErrorCode::NumericalOverflowError)?
                                .checked_div(100)
                                .ok_or(ErrorCode::NumericalOverflowError)?,
                        )
                        .ok_or(ErrorCode::NumericalOverflowError)?,
                    ci_change: 0,
                    new_calculated_divisor: 1,
                    new_calculated_numerator: 1,
                })?,
                _ => return Err(ErrorCode::CannotAlterThisTypeNumerically.into()),
            }
        }
        BasicItemEffectType::IncrementPercentFromBase
        | BasicItemEffectType::DecrementPercentFromBase => match bs.state {
            BasicStatState::Integer { current, .. } => {
                rebalance_basic_stat(RebalanceBasicStatArgs {
                    basic_stat: bs,
                    basic_stat_template: bst,
                    current_change: modded_amount
                        .checked_mul(current)
                        .ok_or(ErrorCode::NumericalOverflowError)?
                        .checked_div(100)
                        .ok_or(ErrorCode::NumericalOverflowError)?,
                    ci_change: 0,
                    new_calculated_divisor: 1,
                    new_calculated_numerator: 1,
                })?
            }
            _ => return Err(ErrorCode::CannotAlterThisTypeNumerically.into()),
        },
    }
    Ok(())
}

pub struct RebalanceStatTemporarilyArgs<'a> {
    pub bie: &'a BasicItemEffect,
    pub bs: &'a mut BasicStat,
    pub bst: &'a BasicStatTemplate,
    pub modded_amount: i64,
    pub adding: bool,
}

pub fn rebalance_stat_temporarily(args: RebalanceStatTemporarilyArgs) -> Result<()> {
    let RebalanceStatTemporarilyArgs {
        bie,
        bs,
        bst,
        modded_amount,
        adding,
    } = args;
    match bie.item_effect_type {
        BasicItemEffectType::Increment | BasicItemEffectType::Decrement => {
            rebalance_basic_stat(RebalanceBasicStatArgs {
                basic_stat: bs,
                basic_stat_template: bst,
                current_change: 0,
                ci_change: modded_amount,
                new_calculated_divisor: 1,
                new_calculated_numerator: 1,
            })?
        }
        BasicItemEffectType::IncrementPercent | BasicItemEffectType::DecrementPercent => {
            let mut adjusted_modded = modded_amount;
            if !adding {
                // we dont need the negative sign here, since we're dividing
                adjusted_modded = modded_amount
                    .checked_mul(-1)
                    .ok_or(ErrorCode::NumericalOverflowError)?;
            }
            rebalance_basic_stat(RebalanceBasicStatArgs {
                basic_stat: bs,
                basic_stat_template: bst,
                current_change: 0,
                ci_change: 0,
                new_calculated_numerator: if adding {
                    100i64
                        .checked_add(adjusted_modded)
                        .ok_or(ErrorCode::NumericalOverflowError)?
                } else {
                    100
                },
                new_calculated_divisor: if adding {
                    100
                } else {
                    100i64
                        .checked_add(modded_amount)
                        .ok_or(ErrorCode::NumericalOverflowError)?
                },
            })?
        }
        BasicItemEffectType::IncrementPercentFromBase
        | BasicItemEffectType::DecrementPercentFromBase => match bs.state {
            BasicStatState::Integer { current, .. } => {
                rebalance_basic_stat(RebalanceBasicStatArgs {
                    basic_stat: bs,
                    basic_stat_template: bst,
                    current_change: 0,
                    ci_change: modded_amount
                        .checked_mul(current)
                        .ok_or(ErrorCode::NumericalOverflowError)?
                        .checked_div(100)
                        .ok_or(ErrorCode::NumericalOverflowError)?,
                    new_calculated_divisor: 1,
                    new_calculated_numerator: 1,
                })?
            }
            _ => return Err(ErrorCode::CannotAlterThisTypeNumerically.into()),
        },
    }

    Ok(())
}

pub struct RebalanceBasicStatArgs<'a> {
    basic_stat: &'a mut BasicStat,
    basic_stat_template: &'a BasicStatTemplate,
    current_change: i64,
    ci_change: i64,
    new_calculated_numerator: i64,
    new_calculated_divisor: i64,
}
pub fn rebalance_basic_stat(args: RebalanceBasicStatArgs) -> Result<()> {
    let RebalanceBasicStatArgs {
        basic_stat,
        basic_stat_template,
        current_change,
        ci_change,
        new_calculated_numerator,
        new_calculated_divisor,
    } = args;
    match basic_stat.state {
        BasicStatState::Integer {
            current,
            calculated_intermediate,
            calculated,
        } => {
            match basic_stat_template.stat_type {
                BasicStatType::Integer { min, max, .. } => {
                    let mut new_current = current
                        .checked_add(current_change)
                        .ok_or(ErrorCode::NumericalOverflowError)?;

                    if let Some(m) = max {
                        new_current = std::cmp::min(m, new_current);
                    }

                    if let Some(m) = min {
                        new_current = std::cmp::max(m, new_current);
                    }

                    let mut new_calculated_intermediate = calculated_intermediate
                        .checked_add(ci_change)
                        .ok_or(ErrorCode::NumericalOverflowError)?;

                    new_calculated_intermediate = new_calculated_intermediate
                        .checked_add(current_change)
                        .ok_or(ErrorCode::NumericalOverflowError)?;

                    if let Some(m) = max {
                        new_calculated_intermediate = std::cmp::min(m, new_calculated_intermediate);
                    }

                    if let Some(m) = min {
                        new_calculated_intermediate = std::cmp::max(m, new_calculated_intermediate);
                    }

                    let mut new_calculated = calculated
                        .checked_mul(new_calculated_intermediate)
                        .ok_or(ErrorCode::NumericalOverflowError)?
                        .checked_div(calculated_intermediate)
                        .ok_or(ErrorCode::NumericalOverflowError)?;

                    new_calculated = new_calculated
                        .checked_mul(new_calculated_numerator)
                        .ok_or(ErrorCode::NumericalOverflowError)?;

                    new_calculated = new_calculated
                        .checked_div(new_calculated_divisor)
                        .ok_or(ErrorCode::NumericalOverflowError)?;

                    if let Some(m) = max {
                        new_calculated = std::cmp::min(m, new_calculated);
                    }

                    if let Some(m) = min {
                        new_calculated = std::cmp::max(m, new_calculated);
                    }

                    basic_stat.state = BasicStatState::Integer {
                        current: new_current,
                        calculated_intermediate: new_calculated_intermediate,
                        calculated: new_calculated,
                    };
                }
                _ => {
                    // Skip, will never be true
                }
            }
        }
        _ => {
            // Do nothing
        }
    }
    Ok(())
}

pub fn recalculate_all_stats(
    player_class: &Account<PlayerClass>,
    player: &mut Account<Player>,
) -> Result<()> {
    if let Some(bsts) = &player_class.data.config.basic_stats {
        if let Some(bss) = &mut player.data.basic_stats {
            for bst in bsts {
                let bs = &mut bss[bst.index as usize];
                rebalance_basic_stat(RebalanceBasicStatArgs {
                    basic_stat: bs,
                    basic_stat_template: bst,
                    current_change: 0,
                    ci_change: 0,
                    new_calculated_divisor: 1,
                    new_calculated_numerator: 1,
                })?
            }
        }
    }

    Ok(())
}

pub struct GetModdedAmountGivenTokensStakedOnItemArgs<'a, 'b, 'info> {
    pub amount: u64,
    pub item: &'b Account<'info, Item>,
    pub bie: &'a BasicItemEffect,
    pub adding: bool,
}
pub fn get_modded_amount_given_tokens_staked_on_item<'a, 'b, 'info>(
    args: GetModdedAmountGivenTokensStakedOnItemArgs<'a, 'b, 'info>,
) -> Result<i64> {
    let GetModdedAmountGivenTokensStakedOnItemArgs {
        amount,
        item,
        bie,
        adding,
    } = args;

    let mut modded_amount: i64 = (amount as i64)
        .checked_mul(bie.amount as i64)
        .ok_or(ErrorCode::NumericalOverflowError)?;

    if item.tokens_staked > 0 {
        let mut to_add: u64 = item.tokens_staked;
        if let Some(san) = bie.staking_amount_numerator {
            to_add = to_add
                .checked_mul(san)
                .ok_or(ErrorCode::NumericalOverflowError)?;
        }
        if let Some(sad) = bie.staking_amount_divisor {
            to_add = to_add
                .checked_div(sad)
                .ok_or(ErrorCode::NumericalOverflowError)?;
        }

        modded_amount = modded_amount
            .checked_add(to_add as i64)
            .ok_or(ErrorCode::NumericalOverflowError)?;
    }

    if bie.item_effect_type == BasicItemEffectType::Decrement
        || bie.item_effect_type == BasicItemEffectType::DecrementPercent
        || bie.item_effect_type == BasicItemEffectType::DecrementPercentFromBase
    {
        modded_amount = modded_amount
            .checked_mul(-1)
            .ok_or(ErrorCode::NumericalOverflowError)?;
    }

    if !adding {
        modded_amount = modded_amount
            .checked_mul(-1)
            .ok_or(ErrorCode::NumericalOverflowError)?;
    }

    Ok(modded_amount)
}
