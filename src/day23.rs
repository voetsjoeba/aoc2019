// vim: set ai et ts=4 sts=4 sw=4:
use crate::util;
use crate::intcode::{CPU, CpuState};

pub fn main() {
    let line: String = util::file_read_lines("input/day23.txt").into_iter().next().unwrap();
    let program: Vec<i64> = line.split(",").map(|s| s.parse().unwrap()).collect();

    println!("{}", part1(&program));
    println!("{}", part2(&program));
}

struct Packet {
    dest_id: usize,
    x: i64,
    y: i64,
}
fn part1(program: &Vec<i64>) -> i64
{
    #[allow(non_snake_case)]
    let N = 50usize;

    let mut nics: Vec<CPU> = (0..N).map(|id| {
        let mut nic = CPU::new(program);
        nic.run();                                    // kick off the CPU and get it in the running state
        assert!(nic.get_state() == CpuState::WaitIO); // should block try to read its input ID first
        nic.send_input(id as i64);
        nic.step();                                   // consume the ID value
        nic
    }).collect();

    // let all CPUs process further instructions in lockstep, and forward any packets in their output queue
    // to their destination NIC's input queue. whenever one stalls on needing input,
    // feed -1 to its input queue and make it re-process the last instruction (which must be an input,
    // because output is already non-blocking).
    loop
    {
        for nic in &mut nics {
            nic.step();
            if nic.get_state() == CpuState::WaitIO {
                nic.send_input(-1);
                nic.step(); // repeat the same input instruction
                assert!(nic.get_state() != CpuState::WaitIO);
            }
        }

        // collect any packets that the NICs have produced, and forward them onto their destination NIC's input queue
        // to be consumed on the next iteration (need to do this in two stages due to ref/mut ref exclusion rules)
        let mut packets = Vec::<Packet>::with_capacity(N);
        for nic in &mut nics {
            if let Some(bytes) = nic.consume_output_n(3) {
                let packet = Packet {
                    dest_id: bytes[0] as usize,
                    x: bytes[1],
                    y: bytes[2],
                };
                if packet.dest_id == 255 {
                    return packet.y; // termination condition
                }
                packets.push(packet);
            }
        }
        for packet in packets {
            nics[packet.dest_id].send_input(packet.x);
            nics[packet.dest_id].send_input(packet.y);
        }
    }
}

fn part2(program: &Vec<i64>) -> i64
{
    // same as before, but now with an additional NAT packet that gets recorded whenever any NIC
    // sends a packet to address 255, plus a check on every iteration to make the NAT kick in if
    // all NICs are idle (i.e. in the WaitIO state with an empty output buffer).
    //
    // Note: deciding whether a NIC is idle is not as straightforward as just looking at how when
    // all NICs are in WaitIO because we're feeding them -1 every time they stall, which means
    // that we can expect an idle state look more like a loop of:
    //     stall, get fed -1, some logic to handle value -1, decide to try and read another packet,
    //     stall again, repeat.
    //
    // instead, we'll look at the input and output queues of each NIC; if all the output queues
    // have been idle for an extended period of time (and by extension all input queues as well,
    // since packets are transferred from output to input queues), then necessarily no more packets
    // are going in or out of any NICs and the network can be considered idle.
    #[allow(non_snake_case)]
    let N = 50usize;

    let mut nics: Vec<CPU> = (0..N).map(|id| {
        let mut nic = CPU::new(program);
        nic.run();
        assert!(nic.get_state() == CpuState::WaitIO);
        nic.send_input(id as i64);
        nic.step();
        nic
    }).collect();

    let mut idle_counter = 0usize;
    let idle_threshold = 650usize; // trial and error, we don't know how long it takes to produce packets, or how much time elapses before a NIC decides to ping the NAT ...
    let mut nat_packet: Option<Packet> = None; // current packet in the NAT buffer
    let mut nat_last_delivered_packet: Option<Packet> = None; // last packet delivered by the NAT to NIC 0

    loop {
        for nic in &mut nics {
            nic.step();
            if nic.get_state() == CpuState::WaitIO {
                nic.send_input(-1);
                nic.step(); // repeat the same input instruction
                assert!(nic.get_state() != CpuState::WaitIO);
            }
        }

        // collect any packets that the NICs have produced, and forward them onto their destination NIC's input queue
        // to be consumed on the next iteration (need to do this in two stages due to ref/mut ref exclusion rules)
        let mut packets_produced = Vec::<Packet>::with_capacity(N);
        for nic in &mut nics {
            if let Some(bytes) = nic.consume_output_n(3) {
                let packet = Packet {
                    dest_id: bytes[0] as usize,
                    x: bytes[1],
                    y: bytes[2],
                };
                if packet.dest_id == 255 {
                    nat_packet = Some(packet);
                } else {
                    packets_produced.push(packet);
                }
            }
        }
        for packet in &packets_produced {
            nics[packet.dest_id].send_input(packet.x);
            nics[packet.dest_id].send_input(packet.y);
        }

        if packets_produced.len() == 0 && nics.iter().all(|nic| nic.peek_input_first().is_none()) {
            idle_counter += 1;
        } else {
            idle_counter = 0;
        }

        if idle_counter >= idle_threshold {
            if let Some(packet) = nat_packet {
                nics[0].send_input(packet.x);
                nics[0].send_input(packet.y);

                // are we delivering the same Y value as the last time?
                if let Some(ldp) = nat_last_delivered_packet {
                    if packet.y == ldp.y {
                        return packet.y;
                    }
                }
                nat_last_delivered_packet = Some(packet);
            } else {
                panic!("network has been idle for {} cycles but no packet was sent to the NAT yet", idle_counter);
            }
            idle_counter = 0;
            nat_packet = None; // clear NAT buffer
        }
    }
}

