use psodata::map::MapEnemy;
use ::maps::InstanceEnemy;
use ::block::lobbyhandler::event::Event;

pub fn convert_enemy(me: &MapEnemy, episode: u8, event: u16, alt_enemies: bool) -> Vec<InstanceEnemy> {
    let mut ret = Vec::new();
    // TODO Rare enemies
    match me.base {
        0x0040 => {
            // Hildebear, Hildetorr
            debug!("Hildebear/Hildetorr");
            ret.push(InstanceEnemy {
                param_entry: 0x49 + (me.skin & 0x01) as usize,
                rt_entry: 0x01 + (me.skin & 0x01) as usize,
                name: "Hildebear/Hildetorr"
            });
        },
        0x0041 => {
            // Rappies, lots of them
            match episode {
                3 => {
                    if alt_enemies {
                        debug!("Sand/Del Rappy (Desert)");
                        ret.push(InstanceEnemy {
                            param_entry: 0x17 + (me.skin & 0x01) as usize,
                            rt_entry: 0x11 + (me.skin & 0x01) as usize,
                            name: "Sand/Del Rappy (Desert)"
                        })
                    } else {
                        debug!("Sand/Del Rappy (Wilds/Crater)");
                        ret.push(InstanceEnemy {
                            param_entry: 0x05 + (me.skin & 0x01) as usize,
                            rt_entry: 0x11 + (me.skin + 0x01) as usize,
                            name: "Sand/Del Rappy (Wilds/Crater)"
                        });
                    }
                },
                1 => {
                    debug!("Rag/El/Al/Pal Rappy (Ep1)");
                    ret.push(InstanceEnemy {
                        param_entry: 0x18 + (me.skin & 0x01) as usize,
                        rt_entry: 0x11 + (me.skin + 0x01) as usize,
                        name: "Rappy"
                    });
                },
                2 => {
                    let rt = if me.skin & 0x01 > 0 { match event {
                        e if e == Event::Christmas as u16 => {
                            debug!("Saint Rappy");
                            79
                        },
                        e if e == Event::Easter as u16 => {
                            debug!("Egg Rappy");
                            81
                        },
                        e if e == Event::Halloween as u16 => {
                            debug!("Hallo Rappy");
                            80
                        },
                        _ => {
                            debug!("Rag Rappy (Ep2)");
                            51
                        }
                    }} else {0x05};
                    ret.push(InstanceEnemy {
                        param_entry: 0x18 + (me.skin & 0x01) as usize,
                        rt_entry: rt,
                        name: "Rappy"
                    })

                },
                _ => unreachable!()
            }
        },
        0x0042 => {
            debug!("Monest and Mothmants");
            ret.push(InstanceEnemy {
                param_entry: 0x01,
                rt_entry: 0x04,
                name: "Monest"
            });
            for _ in 0..30 {
                ret.push(InstanceEnemy {
                    param_entry: 0x00,
                    rt_entry: 0x03,
                    name: "Mothmant"
                });
            }
        },
        0x0043 => {
            debug!("Savage/Barbarous Wolf");
            let acc = if me.reserved2[10] & 0x800000 > 0 {1} else {0};
            ret.push(InstanceEnemy {
                param_entry: 0x02 + acc,
                rt_entry: 0x07 + acc,
                name: "Savage/Barbarous Wolf"
            });
        },
        0x0044 => {
            debug!("Booma");
            ret.push(InstanceEnemy {
                param_entry: 0x48 + (me.skin % 3) as usize,
                rt_entry: 0x09 + (me.skin % 3) as usize,
                name: "Booma"
            });
        },
        0x0060 => {
            debug!("Grass Assassin");
            ret.push(InstanceEnemy {
                param_entry: 0x4E,
                rt_entry: 0x0C,
                name: "Grass Assassin"
            });
        },
        0x0061 => {
            if episode == 2 && alt_enemies {
                debug!("Del Lily (Ep2 Tower)");
                ret.push(InstanceEnemy {
                    param_entry: 0x25,
                    rt_entry: 0x53,
                    name: "Del Lily"
                });
            } else {
                debug!("Poison/Nar Lily");
                let acc = if me.reserved2[10] & 0x800000 > 0 {1} else {0};
                ret.push(InstanceEnemy {
                    param_entry: 0x04 + acc,
                    rt_entry: 0x0D + acc,
                    name: "Poison/Nar Lily"
                });
            }
        },
        0x0062 => {
            debug!("Nano Dragon");
            ret.push(InstanceEnemy {
                param_entry: 0x1A,
                rt_entry: 0x0F,
                name: "Nano Dragon"
            });
        },
        0x0063 => {
            debug!("Shark");
            let acc = me.skin % 3;
            ret.push(InstanceEnemy {
                param_entry: 0x4F + acc as usize,
                rt_entry: 0x10 + acc as usize,
                name: "Shark"
            });
        },
        0x0064 => {
            debug!("Slime and 4 clones");
            let acc = if me.reserved2[10] & 0x800000 > 0 {1} else {0};
            ret.push(InstanceEnemy {
                param_entry: 0x30 - acc,
                rt_entry: 0x13 + acc,
                name: "Slime"
            });
            for _ in 0..4 {
                ret.push(InstanceEnemy {
                    param_entry: 0x30,
                    rt_entry: 0x13,
                    name: "Slime"
                });
            }
        },
        0x0065 => {
            debug!("Pan Arms");
            for i in 0..3 {
                ret.push(InstanceEnemy {
                    param_entry: 0x31 + i,
                    rt_entry: 0x15 + i,
                    name: "Pan Arms"
                });
            }
        },
        0x0080 => {
            debug!("Dubchic/Gilchic");
            let acc = me.skin & 0x01;
            ret.push(InstanceEnemy {
                param_entry: 0x1B + acc as usize,
                rt_entry: 0x18 + acc as usize,
                name: "Dubchic/Gilchic"
            });
        },
        0x0081 => {
            debug!("Garanz");
            ret.push(InstanceEnemy {
                param_entry: 0x1D,
                rt_entry: 0x19,
                name: "Garanz"
            });
        },
        0x0082 => {
            debug!("Sinow Beat/Gold");
            let pe;
            let rt;
            if me.reserved2[10] & 0x800000 > 0 {
                pe = 0x13;
                rt = 0x1B;
            } else {
                pe = 0x06;
                rt = 0x1A;
            }

            for _ in 0..me.num_clones {
                ret.push(InstanceEnemy {
                    param_entry: pe,
                    rt_entry: rt,
                    name: "Sinow Beat/Gold"
                });
            }
        },
        0x0083 => {
            debug!("Canadine (Solo)");
            ret.push(InstanceEnemy {
                param_entry: 0x07,
                rt_entry: 0x1C,
                name: "Canadine (Solo)"
            });
        },
        0x0084 => {
            debug!("Canadine (Squad)");
            ret.push(InstanceEnemy {
                param_entry: 0x09,
                rt_entry: 0x1D,
                name: "Canadine Leader"
            });
            for _ in 0..8 {
                ret.push(InstanceEnemy {
                    param_entry: 0x08,
                    rt_entry: 0x1C,
                    name: "Canadine Squadmate"
                });
            }
        },
        0x0085 => {
            debug!("Dubwitch");
            ret.push(InstanceEnemy {
                param_entry: 0xFFFFFFFF,
                rt_entry: 0xFFFFFFFF,
                name: "Dubwitch"
            });
        },
        0x00A0 => {
            debug!("Delsaber");
            ret.push(InstanceEnemy {
                param_entry: 0x52,
                rt_entry: 0x1E,
                name: "Delsaber"
            });
        },
        0x00A1 => {
            debug!("Chaos Sorcerer + Bee L, Bee R");
            ret.push(InstanceEnemy {
                param_entry: 0x0A,
                rt_entry: 0x1F,
                name: "Chaos Sorcerer"
            });
            ret.push(InstanceEnemy {
                param_entry: 0x0B,
                rt_entry: 0x00,
                name: "Bee L"
            });
            ret.push(InstanceEnemy {
                param_entry: 0x0C,
                rt_entry: 0x00,
                name: "Bee R"
            });
        },
        0x00A2 => {
            debug!("Dark Gunner");
            ret.push(InstanceEnemy {
                param_entry: 0x1E,
                rt_entry: 0x22,
                name: "Dark Gunner"
            });
        },
        0x00A3 => {
            debug!("Death Gunner");
            ret.push(InstanceEnemy {
                param_entry: 0xFFFFFFFF,
                rt_entry: 0xFFFFFFFF,
                name: "Death Gunner (INVALID)"
            })
        },
        0x00A4 => {
            debug!("Chaos Bringer");
            ret.push(InstanceEnemy {
                param_entry: 0x0D,
                rt_entry: 0x24,
                name: "Chaos Bringer"
            });
        },
        0x00A5 => {
            debug!("Dark Belra");
            ret.push(InstanceEnemy {
                param_entry: 0x0E,
                rt_entry: 0x25,
                name: "Dark Belra"
            });
        },
        0x00A6 => {
            debug!("Dimenian");
            let acc = me.skin % 3;
            ret.push(InstanceEnemy {
                param_entry: 0x53 + acc as usize,
                rt_entry: 0x29 + acc as usize,
                name: "Dimenian"
            });
        },
        0x00A7 => {
            debug!("Bulclaw and 4 Claws");
            ret.push(InstanceEnemy {
                param_entry: 0x1F,
                rt_entry: 0x28,
                name: "Bulclaw"
            });
            for _ in 0..4 {
                ret.push(InstanceEnemy {
                    param_entry: 0x20,
                    rt_entry: 0x26,
                    name: "Claw (Bulclaw minion)"
                });
            }
        },
        0x00A8 => {
            debug!("Claw");
            ret.push(InstanceEnemy {
                param_entry: 0x20,
                rt_entry: 0x26,
                name: "Claw"
            });
        },
        0x00C0 => {
            match episode {
                1 => {
                    debug!("Dragon");
                    ret.push(InstanceEnemy {
                        param_entry: 0x12,
                        rt_entry: 0x2C,
                        name: "Dragon"
                    });
                },
                _ => {
                    debug!("Gal Gryphon");
                    ret.push(InstanceEnemy {
                        param_entry: 0x1E,
                        rt_entry: 0x4D,
                        name: "Gal Gryphon"
                    });
                }
            }
        },
        0x00C1 => {
            debug!("De Rol Le");
            ret.push(InstanceEnemy {
                param_entry: 0x0F,
                rt_entry: 0x2D,
                name: "De Rol Le"
            });
        },
        0x00C2 => {
            debug!("Vol Opt Phase 1 (no entry)");
            ret.push(InstanceEnemy {
                param_entry: 0xFFFFFFFF,
                rt_entry: 0xFFFFFFFF,
                name: "Vol Opt Phase 1 (INVALID)"
            })
        },
        0x00C5 => {
            debug!("Vol Opt Phase 2");
            ret.push(InstanceEnemy {
                param_entry: 0x25,
                rt_entry: 0x2E,
                name: "Vol Opt Phase 2"
            });
        },
        0x00C8 => {
            debug!("Dark Falz (3 phases) and 510 Darvants");
            // darvants (spinny hurt things in pre-fight)
            for _ in 0..510 {
                ret.push(InstanceEnemy {
                    param_entry: 0x35,
                    rt_entry: 0x00,
                    name: "Darvant (Falz Minion)"
                });
            }
            // phase 3
            ret.push(InstanceEnemy {
                param_entry: 0x38,
                rt_entry: 0x2F,
                name: "Dark Falz Phase 3"
            });
            // phase 2
            ret.push(InstanceEnemy {
                param_entry: 0x37,
                rt_entry: 0x2F,
                name: "Dark Falz Phase 2"
            });
            // phase 1
            ret.push(InstanceEnemy {
                param_entry: 0x36,
                rt_entry: 0x2F,
                name: "Dark Falz Phase 1"
            });
        },
        0x00CA => {
            debug!("Olga Flow");
            ret.push(InstanceEnemy {
                param_entry: 0x2C,
                rt_entry: 0x4E,
                name: "Olga Flow"
            });
            for _ in 0..512 {
                ret.push(InstanceEnemy {
                    param_entry: 0xFFFFFFFF,
                    rt_entry: 0xFFFFFFFF,
                    name: "Olga Flow Extra (INVALID)"
                });
            }
        },
        0x00CB => {
            debug!("Barba Ray");
            ret.push(InstanceEnemy {
                param_entry: 0x0F,
                rt_entry: 0x49,
                name: "Barba Ray"
            });
            for _ in 0..47 {
                ret.push(InstanceEnemy {
                    param_entry: 0xFFFFFFFF,
                    rt_entry: 0xFFFFFFFF,
                    name: "Barba Ray Extra (INVALID)"
                });
            }
        },
        0x00CC => {
            debug!("Gol Dragon");
            ret.push(InstanceEnemy {
                param_entry: 0x12,
                rt_entry: 0x4C,
                name: "Gol Dragon"
            });
            for _ in 0..5 {
                ret.push(InstanceEnemy {
                    param_entry: 0xFFFFFFFF,
                    rt_entry: 0xFFFFFFFF,
                    name: "Barba Ray Extra (INVALID)"
                });
            }
        },
        0x00D4 => {
            if me.skin > 0 {
                debug!("Sinow Spigell");
                ret.push(InstanceEnemy {
                    param_entry: 0x13,
                    rt_entry: 0x3F,
                    name: "Sinow Spigell"
                });
            } else {
                debug!("Sinow Berill");
                ret.push(InstanceEnemy {
                    param_entry: 0x06,
                    rt_entry: 0x3E,
                    name: "Sinow Berill"
                });
            }
            for _ in 0..4 {
                ret.push(InstanceEnemy {
                    param_entry: 0xFFFFFFFF,
                    rt_entry: 0xFFFFFFFF,
                    name: "Unused (INVALID)"
                });
            }
        },
        0x00D5 => {
            debug!("Merillia and Meriltas");
            ret.push(InstanceEnemy {
                param_entry: 0x4B + (me.skin & 0x01) as usize,
                rt_entry: 0x34 + (me.skin & 0x01) as usize,
                name: "Merillia/Meriltas"
            });
        },
        0x00D6 => {
            match me.skin % 3 {
                2 => {
                    debug!("Mericarol");
                    ret.push(InstanceEnemy {
                        param_entry: 0x44 + 2,
                        rt_entry: 0x38 + 2,
                        name: "Mericarol"
                    });
                },
                1 => {
                    debug!("Merikle");
                    ret.push(InstanceEnemy {
                        param_entry: 0x44 + 1,
                        rt_entry: 0x38 + 1,
                        name: "Merikle"
                    });
                },
                _ => {
                    debug!("Mericus");
                    ret.push(InstanceEnemy {
                        param_entry: 0x3A,
                        rt_entry: 0x38,
                        name: "Mericus"
                    });
                }
            }
        },
        0x00D7 => {
            debug!("Gibbon");
            ret.push(InstanceEnemy {
                param_entry: 0x3B + (me.skin & 0x01) as usize,
                rt_entry: 0x3B + (me.skin & 0x01) as usize,
                name: "Gibbon"
            });
        },
        0x00D8 => {
            debug!("Gibbles");
            ret.push(InstanceEnemy {
                param_entry: 0x3D,
                rt_entry: 0x3D,
                name: "Gibbles"
            });
        },
        0x00D9 => {
            debug!("Gee");
            ret.push(InstanceEnemy {
                param_entry: 0x07,
                rt_entry: 0x36,
                name: "Gee"
            });
        },
        0x00DA => {
            debug!("Gi Gue");
            ret.push(InstanceEnemy {
                param_entry: 0x1A,
                rt_entry: 0x37,
                name: "Gi Gue"
            });
        },
        0x00DB => {
            debug!("Deldepth");
            ret.push(InstanceEnemy {
                param_entry: 0x30,
                rt_entry: 0x47,
                name: "Deldepth"
            });
        },
        0x00DC => {
            debug!("Delbiter");
            ret.push(InstanceEnemy {
                param_entry: 0x0D,
                rt_entry: 0x48,
                name: "Delbiter"
            });
        },
        0x00DD => {
            debug!("Dolmolm and Dolmdarl");
            ret.push(InstanceEnemy {
                param_entry: 0x4F + (me.skin & 0x01) as usize,
                rt_entry: 0x40 + (me.skin & 0x01) as usize,
                name: "Dolmolm/Domldarl"
            });
        },
        0x00DE => {
            debug!("Morfos");
            ret.push(InstanceEnemy {
                param_entry: 0x41,
                rt_entry: 0x42,
                name: "Morfos"
            });
        },
        0x00DF => {
            debug!("Recobox and {} Recons", me.num_clones);
            ret.push(InstanceEnemy {
                param_entry: 0x41,
                rt_entry: 0x43,
                name: "Recobox"
            });
            for _ in 0..me.num_clones {
                ret.push(InstanceEnemy {
                    param_entry: 0x42,
                    rt_entry: 0x44,
                    name: "Recon (Recobox Minion)"
                });
            }
        },
        0x00E0 => {
            debug!("Epsilon, Sinow Zoa, or Zele");
            if episode == 2 && alt_enemies {
                ret.push(InstanceEnemy {
                    param_entry: 0x23,
                    rt_entry: 0x54,
                    name: "Zele"
                });
                for _ in 0..4 {
                    // empty space, out of control!
                    ret.push(InstanceEnemy {
                        param_entry: 0xFFFFFFFF,
                        rt_entry: 0xFFFFFFFF,
                        name: "Empty"
                    })
                }
            } else {
                ret.push(InstanceEnemy {
                    param_entry: 0x43 + (me.skin & 0x01) as usize,
                    rt_entry: 0x45 + (me.skin & 0x01) as usize,
                    name: "Epsilon/Sinow Zoa"
                });
            }
        },
        0x00E1 => {
            debug!("Ill Gill");
            ret.push(InstanceEnemy {
                param_entry: 0x26,
                rt_entry: 0x52,
                name: "Ill Gill"
            });
        },
        0x0110 => {
            debug!("Astark");
            ret.push(InstanceEnemy {
                param_entry: 0x09,
                rt_entry: 0x01,
                name: "Astark"
            });
        },
        0x0111 => {
            let acc = if me.reserved2[10] & 0x800000 > 0 {1} else {0};
            if acc == 1 {
                debug!("Yowie");
            } else {
                debug!("Satellite Lizard");
            }
            let bp = if alt_enemies {
                0x0D + acc + 0x10
            } else {
                0x0D + acc
            };
            ret.push(InstanceEnemy {
                param_entry: bp as usize,
                rt_entry: 0x02 + acc,
                name: "Satellite Lizard/Yowie"
            });
        },
        0x0112 => {
            debug!("Merissa");
            ret.push(InstanceEnemy {
                param_entry: 0x19 + (me.skin & 0x01) as usize,
                rt_entry: 0x04 + (me.skin & 0x01) as usize,
                name: "Merissa"
            });
        },
        0x0113 => {
            debug!("Girtablulu");
            ret.push(InstanceEnemy {
                param_entry: 0x1F,
                rt_entry: 0x06,
                name: "Girtablulu"
            });
        },
        0x0114 => {
            debug!("Zu/Pazuzu");
            let acc = me.skin & 0x01;
            let bp = if alt_enemies {
                0x07 + acc + 0x14
            } else {
                0x07 + acc
            };
            ret.push(InstanceEnemy {
                param_entry: bp as usize,
                rt_entry: 0x07 + acc as usize,
                name: "Zu/Pazuzu"
            });
        },
        0x0115 => {
            debug!("Boota");
            let acc = me.skin % 3;
            let rt = 0x09 + acc;
            let bp = if me.skin & 0x02 > 0 {0x03} else {acc};
            ret.push(InstanceEnemy {
                param_entry: bp as usize,
                rt_entry: rt as usize,
                name: "Boota"
            });
        },
        0x0116 => {
            debug!("Dorphon/Eclair");
            let acc = me.skin & 0x01;
            ret.push(InstanceEnemy {
                param_entry: 0x0F + acc as usize,
                rt_entry: 0x0C + acc as usize,
                name: "Dorphon/Eclair"
            });
        },
        0x0117 => {
            debug!("Goran");
            let acc = me.skin & 0x03;
            let bp = 0x11 + acc;
            let rt = if me.skin & 0x02 > 0 {
                0x0F
            } else if me.skin & 0x01 > 0 {
                0x10
            } else {
                0x0E
            };
            ret.push(InstanceEnemy {
                param_entry: bp as usize,
                rt_entry: rt,
                name: "Goran"
            });
        },
        0x0119 => {
            debug!("Saint Milion, Shambertin or Kondrieu");
            let acc = me.skin & 0x01;
            let bp = 0x22;
            let rt = if me.reserved2[10] & 0x800000 > 0 {0x15} else {0x13 + acc};
            ret.push(InstanceEnemy {
                param_entry: bp,
                rt_entry: rt as usize,
                name: "Saint Milion/Shambertin/Kondrieu"
            });
        },
        _ => {
            ret.push(InstanceEnemy {
                param_entry: 0xFFFFFFFF,
                rt_entry: 0xFFFFFFFF,
                name: "Unknown"
            });
            debug!("Unknown enemy base {}", me.base);
            // So, the official map data has some unknown data at the top of
            // the enemy lists, and they aren't actually counted. We'll just
            // ignore them for now
        }
    };
    ret
}
