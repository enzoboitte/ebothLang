#!/bin/bash

set -e

l_sAsmFile="${1:-main.asm}"
l_sBuildDir="build"
l_sBaseName=$(basename "$l_sAsmFile" .asm)
l_sObjectFile="$l_sBuildDir/$l_sBaseName.o"
l_sExecutable="$l_sBuildDir/$l_sBaseName"

if ! command -v nasm &> /dev/null; then
    echo "Installation de NASM..."
    if command -v apt &> /dev/null; then
        sudo apt update && sudo apt install -y nasm
    elif command -v dnf &> /dev/null; then
        sudo dnf install -y nasm
    elif command -v pacman &> /dev/null; then
        sudo pacman -S --noconfirm nasm
    else
        echo "Gestionnaire de paquets non supporté"
        exit 1
    fi
fi

if ! command -v ld &> /dev/null; then
    echo "Installation de binutils..."
    if command -v apt &> /dev/null; then
        sudo apt install -y binutils
    elif command -v dnf &> /dev/null; then
        sudo dnf install -y binutils
    elif command -v pacman &> /dev/null; then
        sudo pacman -S --noconfirm binutils
    fi
fi

mkdir -p "$l_sBuildDir"

#echo "Assemblage: $l_sAsmFile -> $l_sObjectFile" without warnings
nasm -f elf64 -o "$l_sObjectFile" "$l_sAsmFile" #2>/dev/null

#echo "Linkage: $l_sObjectFile -> $l_sExecutable"
ld "$l_sObjectFile" -o "$l_sExecutable"

#echo "Exécution: $l_sExecutable"
"$l_sExecutable"
#echo -e "Code de sortie: $?"
