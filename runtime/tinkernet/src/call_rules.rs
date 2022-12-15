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
);
