# convert complex xml format to custom easy-scanf txt format
require 'nokogiri'

doc = Nokogiri::XML(ARGF.read)

ElementTypes = %w[ CellTransmitter ObjectiveCrossedWires ObjectiveSignalCount ObjectiveTargetValue PlacedSignal RadialTransmitter Receiver SignalBlock SignalBlockCircle SignalBlockHexagon SignalBooster SwapperTransmitter Transceiver Transmitter ]
ElementGroups = %w[ Cable Exchange Fibre Wave ]
# Exchange: white, Wave: orange

@id = -1
@id_seen = {}

def get_id(old_id)
  @id_seen[old_id.to_s] ||= (@id += 1)
end

def process_element(x)
  id = get_id(x['id'])
  pos = x['position'].sub(/,0$/, '')
  type = ElementTypes.find_index(x['type']) || -1
  color = ElementGroups.find_index(x['elementGroup'] || x['blockGroup']) || -1
  amount = x['amount'] || -1
  target = x['target'] || -1
  
  s = "#{id} #{type} #{pos} ".ljust(30)
  case x['type']
  when 'Receiver'
    s += "#{color} #{target}"
  when 'Transceiver'
    s += "#{color} #{target} #{amount}"
  when 'Transmitter'
    s += "#{color} #{amount}"
  when 'PlacedSignal'
    # ignore
    return
  when 'RadialTransmitter'
    s += "#{color} #{x['minRadius']}" # ignored properties: targetAmount (always big enough), extraRadius (always 0)
  when 'SignalBlock'
    s += "#{color} #{x['sx']} #{x['sy']} #{x['ex']} #{x['ey']}"
  when 'SignalBlockCircle'
    s += "#{color} #{x['radius']}"
  when 'SwapperTransmitter'
    color1 = ElementGroups.find_index(x['swapGroup1'])
    color2 = ElementGroups.find_index(x['swapGroup2'])
    s += "#{color1} #{color2} #{target} #{amount}" # ignored properties: elementGroup, transmitterGroup (always Cable)
  when 'CellTransmitter'
    s += "#{color}"
  when 'SignalBlockHexagon'
    s += "#{color} #{x['radius']} #{x['flip'] == 'True' ? 1 : 0}"
  when 'SignalBooster'
    s += "#{color}"
  when 'ObjectiveSignalCount'
    s += "#{x['signalTarget']}"
  when 'ObjectiveTargetValue'
    s += "#{get_id(x['informationTarget'])}"
  when 'ObjectiveCrossedWires'
    ;
  else
    raise "NotImplemented: #{x['type']}"
  end
  puts "#{s.ljust(42)}  # #{x}"
end

def useful_element?(x)
  !x['type'].include?('Objective') && !x['type'].include?('Block')
end

doc.css('element').select(&method(:useful_element?)).each(&method(:process_element))
doc.css('element').reject(&method(:useful_element?)).each(&method(:process_element))

