class A
  def pub1; end
  private
  def prv1; end
  public
  def pub2; end
  private :pub2
  def self.cls; end
  private_class_method :cls
  protected
  def prot1; end
  class << self
    def meta; end
  end
end
module M
  module_function
  def mf; end
end
x = [1, 2].map { |v| v * 2 }.select do |v| v > 1 end
Foo::Bar.new.baz
