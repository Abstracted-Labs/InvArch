use super::*;
use pallet_rules::build_call_rules;

build_call_rules!(
    Call,

    pallet_balances::pallet,
    Balances {
        transfer {
            dest: <<Runtime as frame_system::Config>::Lookup as StaticLookup>::Source,
            value: <Runtime as pallet_balances::Config>::Balance,
        },
        set_balance {
            who: <<Runtime as frame_system::Config>::Lookup as StaticLookup>::Source,
            new_free: <Runtime as pallet_balances::Config>::Balance,
            new_reserved: <Runtime as pallet_balances::Config>::Balance,
        },
        force_transfer {
            source: <<Runtime as frame_system::Config>::Lookup as StaticLookup>::Source,
            dest: <<Runtime as frame_system::Config>::Lookup as StaticLookup>::Source,
            value: <Runtime as pallet_balances::Config>::Balance,
        },
        transfer_keep_alive {
            dest: <<Runtime as frame_system::Config>::Lookup as StaticLookup>::Source,
            value: <Runtime as pallet_balances::Config>::Balance,
        },
        transfer_all {
            dest: <<Runtime as frame_system::Config>::Lookup as StaticLookup>::Source,
            keep_alive: bool,
        },
        force_unreserve {
            who: <<Runtime as frame_system::Config>::Lookup as StaticLookup>::Source,
            amount: <Runtime as pallet_balances::Config>::Balance,
        },
    },

    orml_vesting::module,
    Vesting {
       // claim {}
        vested_transfer {
            dest: <<Runtime as frame_system::Config>::Lookup as StaticLookup>::Source,
            schedule: orml_vesting::VestingSchedule<
                    <Runtime as frame_system::Config>::BlockNumber,
                <<Runtime as orml_vesting::Config>::Currency as Currency<
                        <Runtime as frame_system::Config>::AccountId,
                    >>::Balance,
                >,
        },
        update_vesting_schedules {
            who: <<Runtime as frame_system::Config>::Lookup as StaticLookup>::Source,
            vesting_schedules: Vec<
                orml_vesting::VestingSchedule<
                        <Runtime as frame_system::Config>::BlockNumber,
                    <<Runtime as orml_vesting::Config>::Currency as Currency<
                            <Runtime as frame_system::Config>::AccountId,
                        >>::Balance,
                    >,
            >,
        },
    },

    frame_system::pallet,
    System {
        remark { remark: Vec<u8> },
    },

    pallet_inv4::pallet,
    INV4 {
        create_ips {
        metadata: Vec<u8>,
        assets: Vec<pallet_inv4::AnyIdOf<Runtime>>,
        allow_replica: bool,
        ipl_license: <Runtime as pallet_inv4::Config>::Licenses,
        ipl_execution_threshold: invarch_primitives::OneOrPercent,
        ipl_default_asset_weight: invarch_primitives::OneOrPercent,
        ipl_default_permission: bool,
    },
    append {
        ips_id: <Runtime as pallet_inv4::Config>::IpId,
        original_caller: Option<<Runtime as frame_system::Config>::AccountId>,
        assets: Vec<pallet_inv4::AnyIdOf<Runtime>>,
        new_metadata: Option<Vec<u8>>,
    },
    remove {
        ips_id: <Runtime as pallet_inv4::Config>::IpId,
        original_caller: Option<<Runtime as frame_system::Config>::AccountId>,
        assets: Vec<pallet_inv4::AnyIdWithNewOwner<Runtime>>,
        new_metadata: Option<Vec<u8>>,
    },
    allow_replica {
        ips_id: <Runtime as pallet_inv4::Config>::IpId,
    },
    disallow_replica {
        ips_id: <Runtime as pallet_inv4::Config>::IpId,
    },
    ipt_mint {
        ipt_id: (<Runtime as pallet_inv4::Config>::IpId, Option<<Runtime as pallet_inv4::Config>::IpId>),
        amount: <Runtime as pallet_inv4::Config>::Balance,
        target: <Runtime as frame_system::Config>::AccountId,
    },
    ipt_burn {
        ipt_id: (<Runtime as pallet_inv4::Config>::IpId, Option<<Runtime as pallet_inv4::Config>::IpId>),
        amount: <Runtime as pallet_inv4::Config>::Balance,
        target: <Runtime as frame_system::Config>::AccountId,
    },
    operate_multisig {
        include_caller: bool,
        ipt_id: (<Runtime as pallet_inv4::Config>::IpId, Option<<Runtime as pallet_inv4::Config>::IpId>),
        metadata: Option<Vec<u8>>,
        call: Box<<Runtime as pallet_inv4::Config>::Call>,
    },
    vote_multisig {
        ipt_id: (<Runtime as pallet_inv4::Config>::IpId, Option<<Runtime as pallet_inv4::Config>::IpId>),
        call_hash: [u8; 32],
    },
    withdraw_vote_multisig {
        ipt_id: (<Runtime as pallet_inv4::Config>::IpId, Option<<Runtime as pallet_inv4::Config>::IpId>),
        call_hash: [u8; 32],
    },
    create_sub_token {
        ips_id: <Runtime as pallet_inv4::Config>::IpId,
        sub_tokens: pallet_inv4::ipt::SubAssetsWithEndowment<Runtime>,
    },
    set_sub_token_weight {
        ips_id: <Runtime as pallet_inv4::Config>::IpId,
        sub_token_id: <Runtime as pallet_inv4::Config>::IpId,
        voting_weight: invarch_primitives::OneOrPercent,
    }
    }
);
