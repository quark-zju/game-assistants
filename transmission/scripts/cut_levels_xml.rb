@buf = []
@level_name = nil
@worlds = File.read('../orig-data/worlds.xml')

require 'fileutils'
FileUtils.mkdir_p('../levels/data')

def flush
  return unless @level_name
  File.write("../levels/data/#{@level_name}.xml", @buf.join("\n"))

  world_line = @worlds.lines.find{|l| l.include?("\"#{@level_name}\"") }
  alt_name = world_line && world_line[/name="([^"]*)"/, 1]
  if alt_name
    File.symlink("data/#{@level_name}.xml", "../levels/#{alt_name}.xml") rescue nil
  end
  @level_name = nil
  @buf = []
end

File.read('../orig-data/levels.xml').lines.each do |l|
  l.chomp!
  name = l[/<!-- (.*) -->/, 1]
  if name
    flush
    @level_name = name
  end
  @buf << l
end

flush
