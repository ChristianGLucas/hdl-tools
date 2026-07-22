// Shared HDL source fixtures for hdl-tools node tests. Included by each
// `nodes/<node>_test.rs` via `#[path = "fixtures.rs"] mod fixtures;`. Every
// expected value asserted against these fixtures in the *_test.rs files was
// hand-derived by reading the fixture text against the IEEE 1800-2017 /
// IEEE 1076 (VHDL) language semantics directly — an oracle independent of
// this package's own extraction code.
#![allow(dead_code)]

/// ANSI-style SystemVerilog module: two parameters (one untyped, one typed
/// `integer`), three ports (two scalar `wire` inputs, one `reg [WIDTH-1:0]`
/// output), a leading module doc comment, and two trailing port doc
/// comments.
pub const VERILOG_COUNTER: &str = "\
// A simple up counter with synchronous reset.
module counter #(
    parameter WIDTH = 8,
    parameter integer STEP = 1
) (
    input  wire             clk,   // system clock
    input  wire             rst_n, // active-low reset
    output reg  [WIDTH-1:0] count
);
  always @(posedge clk) begin
    if (!rst_n)
      count <= 0;
    else
      count <= count + STEP;
  end
endmodule
";

/// Classic non-ANSI Verilog module: ports declared in the header by name
/// only, direction/type/width declared separately in the body.
pub const VERILOG_NONANSI: &str = "\
module adder(a, b, sum);
  input [3:0] a;
  input [3:0] b;
  output [4:0] sum;
  assign sum = a + b;
endmodule
";

/// Two independent modules in one file, to exercise multi-definition
/// extraction (ListDefinitions, ParseVerilogModules module_count).
pub const VERILOG_TWO_MODULES: &str = "\
module first_mod(input a, output b);
endmodule

module second_mod(input x, output y);
endmodule
";

/// Deliberately malformed Verilog: a module missing its closing `endmodule`
/// and with an unterminated port list.
pub const VERILOG_MALFORMED: &str = "module broken #(parameter W = ( (input a\n";

/// ANSI-style VHDL entity: two generics (both `integer` with defaults),
/// three ports (two scalar `std_logic` inputs, one `std_logic_vector` output
/// with a `downto` range), a leading entity doc comment, and two trailing
/// port doc comments.
pub const VHDL_COUNTER: &str = "\
library ieee;
use ieee.std_logic_1164.all;

-- A simple up counter with synchronous reset.
entity counter is
  generic (
    WIDTH : integer := 8;
    STEP  : integer := 1
  );
  port (
    clk   : in  std_logic;                          -- system clock
    rst_n : in  std_logic;                           -- active-low reset
    count : out std_logic_vector(WIDTH-1 downto 0)
  );
end entity counter;

architecture rtl of counter is
begin
end architecture rtl;
";

/// A VHDL package with a constant, an enumeration type, a subtype, a
/// function signature, and a procedure signature (with a `signal` parameter
/// and an `inout` parameter).
pub const VHDL_PACKAGE: &str = "\
package math_pkg is
  constant MAX_WIDTH : integer := 32;
  type state_t is (IDLE, RUN, DONE);
  subtype byte_t is std_logic_vector(7 downto 0);

  function clamp(x : integer; lo : integer; hi : integer) return integer;
  procedure pulse(signal clk : in std_logic; count : inout integer);
end package math_pkg;
";

/// Deliberately malformed VHDL: an entity with an unterminated generic
/// clause and no `end entity`.
pub const VHDL_MALFORMED: &str = "entity broken is\n  generic (\n    W : integer :=\n";

/// A found-by-the-adversarial-reviewer regression case: an otherwise-valid,
/// findable VHDL entity ("top" parses and IS found by name) but with a
/// missing semicolon between its two port declarations. tree-sitter's error
/// recovery absorbs the second port into an ERROR node, which silently
/// flips the first port's reported direction and drops the second port
/// entirely — has_error on the returned module/port-list is what lets a
/// caller detect that the extracted ports are not trustworthy here.
pub const VHDL_PORT_LIST_MISSING_SEMICOLON: &str = "\
entity top is
  port (
    a : in std_logic
    b : out std_logic
  );
end entity top;
";
