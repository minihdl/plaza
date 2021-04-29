extern crate plaza;

use std::io;

fn main() {
    let mut chip = plaza::dev::GAL16V8::new();

    let rin = chip.input(2, "rin");

    let bit3 = chip.input(3, "bit3");
    let bit2 = chip.input(4, "bit2");
    let bit1 = chip.input(5, "bit1");
    let bit0 = chip.input(6, "bit0");

    let mut d0 = !&bit3 & !&bit2 & !&bit1 & !&bit0;
    let     d1 = !&bit3 & !&bit2 & !&bit1 &  &bit0;
    let     d2 = !&bit3 & !&bit2 &  &bit1 & !&bit0;
    let     d3 = !&bit3 & !&bit2 &  &bit1 &  &bit0;
    let     d4 = !&bit3 &  &bit2 & !&bit1 & !&bit0;
    let     d5 = !&bit3 &  &bit2 & !&bit1 &  &bit0;
    let     d6 = !&bit3 &  &bit2 &  &bit1 & !&bit0;
    let     d7 = !&bit3 &  &bit2 &  &bit1 &  &bit0;
    let     d8 =  &bit3 & !&bit2 & !&bit1 & !&bit0;
    let     d9 =  &bit3 & !&bit2 & !&bit1 &  &bit0;
    let     da =  &bit3 & !&bit2 &  &bit1 & !&bit0;
    let     db =  &bit3 & !&bit2 &  &bit1 &  &bit0;
    let     dc =  &bit3 &  &bit2 & !&bit1 & !&bit0;
    let     dd =  &bit3 &  &bit2 & !&bit1 &  &bit0;
    let     de =  &bit3 &  &bit2 &  &bit1 & !&bit0;
    let     df =  &bit3 &  &bit2 &  &bit1 &  &bit0;

    // The rules for rin and rout:
    //   Least significant digit should have rin set high
    //   Most significant digit should have rin set high if you want leading zeroes, rin set low if you want leading blanks
    //   Remaining digits should have rin set to rout from their more-significant neighbor
    // N.B. if you don't ever want leading blanks, you can also just set all rin inputs high

    let rout = &rin | !&d0;

    d0 &= &rin;

    //    #A#               #A#      #A#
    //   #   #        #        #        #
    //   F   B        B        B        B
    //   #   #        #        #        #
    //                      #G#      #G#
    //   #   #        #    #            #
    //   E   C        C    E            C
    //   #   #        #    #            #
    //    #D#               #D#      #D#
    //
    //             #A#      #A#      #A#
    //   #   #    #        #            #
    //   F   B    F        F            B
    //   #   #    #        #            #
    //    #G#      #G#      #G#
    //       #        #    #   #        #
    //       C        C    E   C        C
    //       #        #    #   #        #
    //             #D#      #D#
    //
    //    #A#      #A#      #A#
    //   #   #    #   #    #   #    #
    //   F   B    F   B    F   B    F
    //   #   #    #   #    #   #    #
    //    #G#      #G#      #G#      #G#
    //   #   #        #    #   #    #   #
    //   E   C        C    E   C    E   C
    //   #   #        #    #   #    #   #
    //    #D#                        #D#
    //
    //                      #A#      #A#
    //                #    #        #
    //                B    F        F
    //                #    #        #
    //    #G#      #G#      #G#      #G#
    //   #        #   #    #        #
    //   E        E   C    E        E
    //   #        #   #    #        #
    //    #D#      #D#      #D#

    let sega = &d0       | &d2 | &d3       | &d5 | &d6 | &d7 | &d8 | &d9 | &da                   | &de | &df ;
    let segb = &d0 | &d1 | &d2 | &d3 | &d4             | &d7 | &d8 | &d9 | &da             | &dd             ;
    let segc = &d0 | &d1       | &d3 | &d4 | &d5 | &d6 | &d7 | &d8 | &d9 | &da | &db       | &dd             ;
    let segd = &d0       | &d2 | &d3       | &d5 | &d6       | &d8             | &db | &dc | &dd | &de       ;
    let sege = &d0       | &d2                   | &d6       | &d8       | &da | &db | &dc | &dd | &de | &df ;
    let segf = &d0                   | &d4 | &d5 | &d6       | &d8 | &d9 | &da | &db             | &de | &df ;
    let segg =             &d2 | &d3 | &d4 | &d5 | &d6       | &d8 | &d9 | &da | &db | &dc | &dd | &de | &df ;

    chip.combinatorial_output(12, rout);
    chip.combinatorial_output(13, sega);
    chip.combinatorial_output(14, segb);
    chip.combinatorial_output(15, segc);
    chip.combinatorial_output(16, segd);
    chip.combinatorial_output(17, sege);
    chip.combinatorial_output(18, segf);
    chip.combinatorial_output(19, segg);

    chip.write(&mut io::stdout()).unwrap();
}
