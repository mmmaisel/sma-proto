--[[***************************************************************************]
    sma-proto - A SMA Speedwire protocol library
    Copyright (C) 2024 Max Maisel

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU Affero General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU Affero General Public License for more details.

    You should have received a copy of the GNU Affero General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
[***************************************************************************--]]
sma_protocol = Proto("SMA", "SMA Speedwire")

sma_fourcc = ProtoField.uint32("sma.fourcc", "SMA FourCC", base.HEX)
start_len = ProtoField.uint16("sma.start_len", "Start Tag Length", base.DEC)
start_tag = ProtoField.uint16("sma.start_tag", "Start Tag", base.HEX)
hdr_group = ProtoField.uint32("sma.hdr_group", "Group", base.DEC)
hdr_dlen = ProtoField.uint16("sma.hdr_dlen", "Data Length", base.DEC)
hdr_version = ProtoField.uint16("sma.hdr_version", "Version", base.DEC)
hdr_proto = ProtoField.uint16("sma.hdr_proto", "Protocol", base.HEX)

em_susy_id = ProtoField.uint16("sma.em_susy_id", "SUSy ID", base.HEX)
em_serial = ProtoField.uint32("sma.em_serial", "Serial Number", base.DEC)
em_ctrl = ProtoField.uint16("sma.em_ctrl", "Control", base.HEX)
em_timestamp = ProtoField.uint32("sma.em_timestamp", "Timestamp ms", base.DEC)
em_padding = ProtoField.uint16("sma.em_padding", "Padding", base.HEX)
obis_record = ProtoField.none("sma.obis_number", "Obis Record:")

inv_wordcount = ProtoField.uint8("sma.inv_wordcount", "Word Count", base.DEC)
inv_class = ProtoField.uint8("sma.inv_class", "Class", base.HEX)
inv_ep_susy_id = ProtoField.uint16("sma.inv_ep_susy_id", "SUSy ID", base.HEX)
inv_ep_serial = ProtoField.uint32("sma.inv_ep_serial", "Serial", base.DEC)
inv_ep_ctrl = ProtoField.uint16("sma.inv_ep_ctrl", "Control", base.HEX)
inv_error_code = ProtoField.uint16("sma.inv_error_code", "Error Code", base.DEC)
inv_fragment_id =
    ProtoField.uint16("sma.inv_fragment_id", "Fragment ID", base.DEC)
inv_packet_id = ProtoField.uint16("sma.inv_packet_id", "Packet ID", base.DEC)
inv_channel = ProtoField.uint8("sma.inv_channel", "Channel", base.DEC)
inv_opcode = ProtoField.uint32("sma.inv_opcode", "Opcode", base.HEX)
inv_padding = ProtoField.bytes("sma.padding", "Padding", base.NONE)
inv_identity = ProtoField.bytes("sma.identity", "Identity", base.NONE)

inv_login_user_group =
    ProtoField.uint32("sma.inv_login_user_group", "User Group", base.DEC)
inv_login_timeout =
    ProtoField.uint32("sma.inv_login_timeout", "Timeout", base.DEC)
inv_login_timestamp =
    ProtoField.uint32("sma.inv_login_timestamp", "Timestamp", base.DEC)
inv_login_password =
    ProtoField.bytes("sma.inv_login_password", "Encoded Password", base.NONE)

inv_daydata_start_time =
    ProtoField.uint32("sma.daydata_start_time", "Start Time", base.DEC)
inv_daydata_end_time =
    ProtoField.uint32("sma.daydata_end_time", "End Time", base.DEC)
inv_daydata_start_idx =
    ProtoField.uint32("sma.daydata_start_idx", "Start Index", base.DEC)
inv_daydata_end_idx =
    ProtoField.uint32("sma.daydata_end_idx", "End Index", base.DEC)
inv_daydata_record = ProtoField.none("sma.daydata_record", "Data Point")

sma_end = ProtoField.bytes("sma.end", "End token", base.NONE)

sma_protocol.fields = {
    sma_fourcc,
    start_len,
    start_tag,
    hdr_group,
    hdr_dlen,
    hdr_version,
    hdr_proto,
    em_susy_id,
    em_serial,
    em_ctrl,
    em_timestamp,
    em_padding,
    obis_record,
    inv_wordcount,
    inv_class,
    inv_ep_susy_id,
    inv_ep_serial,
    inv_ep_ctrl,
    inv_error_code,
    inv_fragment_id,
    inv_packet_id,
    inv_channel,
    inv_opcode,
    inv_padding,
    inv_identity,
    inv_login_user_group,
    inv_login_timeout,
    inv_login_timestamp,
    inv_login_password,
    inv_daydata_start_time,
    inv_daydata_end_time,
    inv_daydata_start_idx,
    inv_daydata_end_idx,
    inv_daydata_record,
    sma_end,
}

function protocol_name(proto_id)
    if proto_id == 0x6069 then
        return "(Energy Meter)"
    elseif proto_id == 0x6065 then
        return "(Inverter)"
    elseif proto_id == 0x6081 then
        return "(Extended Engery Meter)"
    else
        return "(Unknown)"
    end
end

function format_obis(obis_channel, obis_meas, obis_type, obis_tariff, val)
    local base_meas = obis_meas % 0x14
    local phase_meas = math.floor(obis_meas / 0x14)
    local unit = ""
    local class = ""

    if base_meas == 1 or base_meas == 2 then
        class = base_meas == 1 and "Active Power +" or "Active Power -"
        if obis_type == 4 then
            unit = "e-1 W"
        else
            unit = " Ws"
        end
    elseif base_meas == 3 or base_meas == 4 then
        class = base_meas == 3 and "Reactive Power +" or "Reactive Power -"
        if obis_type == 4 then
            unit = "e-1 VAr"
        else
            unit = " VArs"
        end
    elseif base_meas == 9 or base_meas == 10 then
        class = base_meas == 9 and "Apparent Power +" or "Apparent Power -"
        if obis_type == 4 then
            unit = "e-1 VA"
        else
            unit = " VAs"
        end
    elseif base_meas == 11 then
        class = "Current"
        unit = " mA"
    elseif base_meas == 12 then
        class = "Voltage"
        unit = " mV"
    elseif base_meas == 13 then
        class = "Power Factor"
        unit = "e-3"
    end

    if obis_channel == 0 and phase_meas == 0 then
        class = "Sum " .. class
    elseif obis_channel == 0 and phase_meas <= 3 then
        class = string.format("L%d ", phase_meas) .. class
    elseif obis_channel == 144 then
        class = "Version"
    else
        class = "Unknown"
    end

    return string.format(
        "%s (%d:%d.%d.%d): %s%s",
        class,
        obis_channel,
        obis_meas,
        obis_type,
        obis_tariff,
        val,
        unit
    )
end

function sma_protocol.dissector(buffer, pinfo, tree)
    length = buffer:len()
    if length == 0 then
        return
    end

    pinfo.cols.protocol = sma_protocol.name

    if buffer(0, 4):uint() ~= 0x534d4100 then
        return
    end
    local dlen = buffer(12, 2):uint()

    local smatree =
        tree:add(sma_protocol, buffer(0, dlen + 20), "SMA Speedwire")
    local hdrtree = smatree:add(sma_protocol, buffer(0, 18), "Header")

    hdrtree:add(sma_fourcc, buffer(0, 4))
    hdrtree:add(start_len, buffer(4, 2))
    hdrtree:add(start_tag, buffer(6, 2))
    hdrtree:add(hdr_group, buffer(8, 4))
    hdrtree:add(hdr_dlen, buffer(12, 2))
    hdrtree:add(hdr_version, buffer(14, 2))
    hdrtree
        :add(hdr_proto, buffer(16, 2))
        :append_text(" " .. protocol_name(buffer(16, 2):uint()))

    if buffer(16, 2):uint() == 0x6069 then
        local emtree = smatree:add(sma_protocol, buffer(18, 10), "Energy Meter")
        emtree:add(em_susy_id, buffer(18, 2))
        emtree:add(em_serial, buffer(20, 4))
        emtree:add(em_timestamp, buffer(24, 4))

        local pos = 12
        local datatree =
            smatree:add(sma_protocol, buffer(28, dlen - 12), "Payload")

        while pos < dlen do
            local obis_channel = buffer(pos + 16, 1):uint()
            local obis_meas = buffer(pos + 17, 1):uint()
            local obis_type = buffer(pos + 18, 1):uint()
            local obis_tariff = buffer(pos + 19, 1):uint()
            local obis_data = 0
            local obis_len = 0

            if obis_type == 8 then
                obis_data = buffer(pos + 20, 8):uint64()
                obis_len = 12
            else
                obis_data = buffer(pos + 20, 4):uint()
                obis_len = 8
            end

            datatree:add(obis_record, buffer(pos + 16, obis_len)):append_text(
                " "
                    .. format_obis(
                        obis_channel,
                        obis_meas,
                        obis_type,
                        obis_tariff,
                        obis_data
                    )
            )
            pos = pos + obis_len
        end
    elseif buffer(16, 2):uint() == 0x6065 then
        local invtree = smatree:add(sma_protocol, buffer(18, 28), "Inverter")
        invtree:add(inv_wordcount, buffer(18, 1))
        invtree:add(inv_class, buffer(19, 1))

        local dsttree =
            invtree:add(sma_protocol, buffer(20, 8), "Destination Endpoint")
        dsttree:add(inv_ep_susy_id, buffer(20, 2))
        dsttree:add(inv_ep_serial, buffer(22, 4))
        dsttree:add(inv_ep_ctrl, buffer(26, 2))

        local apptree =
            invtree:add(sma_protocol, buffer(28, 8), "Application Endpoint")
        apptree:add(inv_ep_susy_id, buffer(28, 2))
        apptree:add(inv_ep_serial, buffer(30, 4))
        apptree:add(inv_ep_ctrl, buffer(34, 2))

        invtree:add(inv_error_code, buffer(36, 2))
        invtree:add_le(inv_fragment_id, buffer(38, 2))
        invtree:add_le(inv_packet_id, buffer(40, 2))
        invtree:add(inv_channel, buffer(42, 1))
        invtree:add(inv_opcode, buffer(43, 3))

        if buffer(43, 3):uint() == 0x20000 then
            local idtree =
                smatree:add(sma_protocol, buffer(46, dlen - 30), "Identify")
            if dlen - 32 <= 8 then
                idtree:add(inv_padding, buffer(46, 8))
            else
                idtree:add(inv_identity, buffer(46, 48))
            end
        elseif buffer(43, 3):uint() == 0x20070 then
            local datatree =
                smatree:add(sma_protocol, buffer(46, dlen - 30), "Get Day Data")
            if dlen <= 58 then
                datatree:add_le(inv_daydata_start_time, buffer(46, 4))
                datatree:add_le(inv_daydata_end_time, buffer(50, 4))
            else
                datatree:add_le(inv_daydata_start_idx, buffer(46, 4))
                datatree:add_le(inv_daydata_end_idx, buffer(50, 4))
            end

            local pos = 54
            while pos < dlen - 30 + 46 do
                datatree:add(inv_daydata_record, buffer(pos, 12)):append_text(
                    string.format(
                        ": Timestamp %d, Energy: %s Wh",
                        buffer(pos, 4):le_uint(),
                        buffer(pos + 4, 8):le_uint64()
                    )
                )
                pos = pos + 12
            end
        elseif buffer(43, 3):uint() == 0x1FDFF then
            local logouttree =
                smatree:add(sma_protocol, buffer(46, dlen - 30), "Logout")
            logouttree:add(inv_padding, buffer(46, 4))
        elseif buffer(43, 3):uint() == 0x4FDFF then
            local logintree =
                smatree:add(sma_protocol, buffer(46, dlen - 30), "Login")
            logintree:add_le(inv_login_user_group, buffer(46, 4))
            logintree:add_le(inv_login_timeout, buffer(50, 4))
            logintree:add_le(inv_login_timestamp, buffer(54, 4))
            logintree:add(inv_padding, buffer(58, 4))
            if dlen - 46 >= 12 then
                logintree:add(inv_login_password, buffer(62, 12))
            end
        end
    elseif buffer(16, 2):uint() == 0x6081 then
        local emtree =
            smatree:add(sma_protocol, buffer(18, 10), "Extended Energy Meter")
        emtree:add_le(em_ctrl, buffer(18, 2))
        emtree:add_le(em_susy_id, buffer(20, 2))
        emtree:add_le(em_serial, buffer(22, 4))
        emtree:add_le(em_padding, buffer(26, 2))
    end

    local endtree = smatree:add(sma_protocol, buffer(dlen + 16), "End token")
    endtree:add(sma_end, buffer(dlen + 16, 4))
end

local udp_port = DissectorTable.get("udp.port")
udp_port:add(9522, sma_protocol)
