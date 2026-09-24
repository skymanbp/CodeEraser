require 'json'
require_relative 'lib/helper'
load 'x.rb'
autoload :Foo, 'foo'
# line comment
=begin
block comment
=end
module Outer
  class Probe < Base
    include Comparable
    attr_reader :x
    def initialize(x) @x = x end
    def self.make(a) new(a) end
    def get; @x; end
    private
    def hidden; end
    protected def prot; end
    private def prv2; end
    private :get
    public
    def loop(n)
      if n > 1 then puts 'x' elsif n == 0 then puts 'y' else puts 'z' end
      puts 'u' unless n
      n.times { |i| next if i == 1; break if i == 2 }
      while n > 0 do n -= 1 end
      until n > 3 do n += 1 end
      for i in 1..3 do end
      case n when 1 then 1 when 2, 3 then 2 else 0 end
      case n
      in Integer then 1
      in [a, b] then 2
      end
      t = n > 0 ? 1 : 0
      begin
        risky
      rescue ArgumentError => e
        retry
      ensure
        cleanup
      end
      risky rescue nil
      i += 1 while i < 3
      private_class_method :make
      define_method(:dyn) { |a| a }
      module_function
      b = n > 0 && n < 5 || n == 9 and n or !n
      x = n if n
      lam = ->(a) { a }
      pr = proc { |a| a }
      get; self.get; obj.get; Probe.make(1); Outer::Probe.make(2); hidden
      "str #{n}" + 'sq' + %q(x) + %Q(y) + `cmd` + :sym.to_s + <<~EOS
        heredoc
      EOS
      loop do break end
      $stdout.puts 42
    end
    def method_missing(name, *args) end
    def <=>(other) end
    def name=(v) end
    def ok?; true; end
  end
  Probe.new(1).get
end
def toplevel; end
