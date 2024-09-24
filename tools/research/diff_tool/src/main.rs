use std::error::Error;

#[derive(Copy, Clone, Debug)]
struct Diff {
    original_byte: u8,
    new_byte: u8,
    offset: usize,
}

fn main() -> Result<(), Box<dyn Error>> {
    let orig = std::fs::read("./original")?;
    let patch = std::fs::read("./modified")?;

    let mut diffs = Vec::new();

    // Check lengths match
    assert_eq!(orig.len(), patch.len());

    // Create patch
    for i in 0..orig.len() {
        let obyte = orig[i];
        let nbyte = patch[i];
        if obyte != nbyte {
            diffs.push(Diff {
                original_byte: obyte,
                new_byte: nbyte,
                offset: i,
            });
        }
    }

    println!("{:?}", diffs.len());


    let mut new_file = orig.clone();

    let mut t = 0;

    // Create a partial application of the patch, to see how much is really needed
    for i in 3..(diffs.len() - 3) {
        let diff = diffs[i];

        println!("Applying {:?}", diff);

        let obyte = new_file[diff.offset];
        // Ensure that the byte we are replacing is the same as what we expect here
        assert_eq!(diff.original_byte, obyte);
        new_file[diff.offset] = diff.new_byte;
           t += 1;
    }
    std::fs::write("./out/IMG_0437.DAT", &new_file)?;
println!("changed = {}", t);

    

    Ok(())
}
