use std::io::{Result, Write, stdout, self};
use std::path::{Path, PathBuf};
use std::fs;

fn main() -> Result<()> {
  let stdout = stdout();
  let mut stdout = stdout.lock();
  for entry in fs::read_dir("/sys/class/power_supply")? {
    let entry = entry?;
    if let Err(e) = process(entry.path(), &mut stdout) {
      writeln!(stdout, "Error {:?}", e)?;
    }
  }

  Ok(())
}

fn get_number_value(p: &Path) -> Option<usize> {
  fs::read_to_string(p).ok().map(|x| x.trim().parse().unwrap())
}

fn process<W: Write>(mut p: PathBuf, f: &mut W) -> Result<()> {
  let name = p.file_name().unwrap().to_string_lossy().into_owned();

  p.push("status");
  let status = match fs::read_to_string(&p) {
    Ok(s) => s,
    Err(e) if e.kind() == io::ErrorKind::NotFound => {
      return Ok(())
    },
    Err(e) => return Err(e),
  };

  write!(f, "{}: ", name)?;

  let status = status.trim();
  f.write_all(status.as_bytes())?;

  p.set_file_name("capacity");
  let percent = get_number_value(&p)
    .map(|x| x as isize).unwrap_or(-1);
  write!(f, ", {}% ", percent)?;

  p.set_file_name("power_now");
  match get_number_value(&p) {
    Some(power_now) if power_now > 0 => {
      p.set_file_name("energy_full");
      let energy_full = get_number_value(&p).unwrap();
      p.set_file_name("energy_now");
      let energy_now = get_number_value(&p).unwrap();

      match status {
        "Discharging" => {
          let t = 3600 * energy_now / power_now;
          write!(f, "{} remaining", show_time(t))?;
        },
        "Charging" => {
          let t = 3600 * (energy_full - energy_now) / power_now;
          write!(f, "{} until charged", show_time(t))?;
        },
        _ => {},
      };
    },
    _ => { }
  }

  f.write_all(b"\n")?;

  Ok(())
}

fn show_time(t: usize) -> String {
  let m = t / 60 % 60;
  let h = t / 3600;
  format!("{}:{:02}", h, m)
}
