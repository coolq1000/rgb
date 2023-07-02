
import json
import re


def transform_operand(operand: str):
    tokens = re.split('([(), +])', operand)

    new_tokens = []

    for token in tokens:
        swapped = ''

        if token == 'af':
            swapped = '%af'
        elif token == 'bc':
            swapped = '%bc'
        elif token == 'de':
            swapped = '%de'
        elif token == 'hl':
            swapped = '%hl'
        elif token == 'pc':
            swapped = '%pc'
        elif token == 'sp':
            swapped = '%sp'
        elif token == 'a':
            swapped = '%a'
        elif token == 'f':
            swapped = '%f'
        elif token == 'b':
            swapped = '%b'
        elif token == 'c':
            swapped = '%c'
        elif token == 'd':
            swapped = '%d'
        elif token == 'e':
            swapped = '%e'
        elif token == 'h':
            swapped = '%h'
        elif token == 'l':
            swapped = '%l'
        else:
            swapped = token

        new_tokens.append(swapped)

    return ''.join(new_tokens)


def output_opcode(opcode):
    disasm = ''

    # account for cb opcode
    length = opcode['length'] if opcode['addr'] != '0xcb' else 2

    if 'operand2' in opcode.keys():
        disasm = f'{opcode["mnemonic"].lower()} {opcode["operand1"].lower()}, {transform_operand(opcode["operand2"].lower())}'
    elif 'operand1' in opcode.keys():
        disasm = f'{opcode["mnemonic"].lower()} {opcode["operand1"].lower()}'
    else:
        disasm = f'{opcode["mnemonic"].lower()}'

    return f'Disasm("{disasm}", {length}),'


with open('scripts/opcodes.json') as f:
    print('// === UNPREFIXED ===')
    opcodes = json.load(f)
    for i in range(0x100):
        if i % 10 == 0:
            print('// ' + hex(i))
        exists = False
        for i_opcode, opcode in opcodes['unprefixed'].items():
            if int(i_opcode, 16) == i:
                exists = True
                print(output_opcode(opcode))
        if not exists:
            print('Disasm("#ud", 1),')
    print('\n// === CBPREFIXED ===')
    for i, opcode in enumerate(list(opcodes['cbprefixed'].values())):
        if i % 10 == 0:
            print('// ' + hex(i))
        print(output_opcode(opcode))
