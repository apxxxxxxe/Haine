----------------------------------------------------------------------
���ukisaragi�v�FMeCab���b�p�[SAORI
��Written by CSaori Project (C.Ponapalt)
�@http://code.google.com/p/csaori/
----------------------------------------------------------------------

������͉���������̂�

�@���{��`�ԑf��̓\�t�g"MeCab"��SAORI�K�iDLL�Ƃ��Ďg����悤�ɂ������̂ł��B
�@
�@���Ɓ[�������mutsuki.dll�������Ǝ�y�Ɉ�����悤�ɉ��ǂ���ړI�Ő��܂�܂����B
�@�r���̎��ł�����A�������́u�@���v�Ƃ����킯�ł��B
�@
�@mutsuki.dll����̈ڍs�́A�����ނ˂��̂܂ܒu�����������ōςނ悤�ɂȂ��Ă��܂��B

�������

�EWin2000�ȏ�

���g�p���@

kisaragi.dll
libmecab.dll
dic�t�H���_�̒��g(�P�ꎫ��)

�ȏ�3�̗v�f�𓯂��t�H���_�̒��ɓ���āAkisaragi.dll��SAORI�Ƃ��ČĂяo���Ă��������B

�S�[�X�g�̈ꕔ�Ƃ��Ĕz�z����ۂ́Alicense.txt���m�F���A���g��readme�Ȃǂɖ��L���Ă��������B

��parse

  ����
    Argument0: parse
    Argument1: ��͂��镶����
    Argument2: �o�̓t�H�[�}�b�g(�ȗ��\)

  �߂�l(Result)
    -1:    �o�̓t�H�[�}�b�g���ُ킾����
    0�ȏ�: �o�͌��ʂ̍s��

  �߂�l(Value*)
    ��͌���

  �@�\
    �������MeCab�ɓn���A�`�ԑf��͂��s���܂��B�o�̓t�H�[�}�b�g��ChaSen�݊��ł��B
    Argument2�ŏo�̓t�H�[�}�b�g���w��ł��܂��B
    �o�͌��ʂ�Value*�ɍs�����Ƃɕ������ĕԂ��܂��B�߂�l�͍s���ł��B

  ��
    Argument0: parse
    Argument1: �@���Ɛ\���܂��B�����ɒu���Ă��������ˁB
    �@��
    Result: 13
    Value0: �@��	�L�T���M	�@��	����-���		
    Value1: ��	�g	��	����-�i����-���p		
    Value2: �\��	���E�V	�\��	����-����	�ܒi�E�T�s	�A�p�`
    Value3: �܂�	�}�X	�܂�	������	����E�}�X	��{�`
    Value4: �B	�B	�B	�L��-��_		
    Value5: ����	�I�\�o	����	����-���		
    Value6: ��	�j	��	����-�i����-���		
    Value7: �u��	�I�C	�u��	����-����	�ܒi�E�J�s�C����	�A�p�^�ڑ�
    Value8: ��	�e	��	����-�ڑ�����		
    Value9: ��������	�N�_�T�C	��������	����-�񎩗�	�ܒi�E���s����	���߂�
    Value10: ��	�l	��	����-�I����		
    Value11: �B	�B	�B	�L��-��_		
    Value12: EOS


��parse-mecab

  parse�Ɠ����ł����A�o�̓t�H�[�}�b�g��MeCab�l�C�e�B�u�ƂȂ�܂��B
  �t�H�[�}�b�g�̈Ⴂ�ɂ��ẮA�Ⴆ�Ή��L���Q�l�ɂȂ�܂��B
  http://d.hatena.ne.jp/Kshi_Kshi/20110102/1293920002


��wakati

  ����
    Argument0: parse
    Argument1: ��͂��镶����

  �߂�l(Result)
    -1:    �o�̓t�H�[�}�b�g���ُ킾����
    0�ȏ�: �o�͌��ʂ̍s��

  �߂�l(Value*)
    ��͌���

  �@�\
    ���p�󔒋�؂�ŕ��������������܂��B

  ��
    Argument0: wakati
    Argument1: ���Č��āA���̋P�����B���͂��A�����Ƌ߂��Ō��Ă�B
    �@��
    Result: 1
    Value0: �� �� �� �� �A ���� �P�� �� �B �� �͂� �A ������ �߂� �� �� �� �� �B 


��yomi

  ����
    Argument0: yomi
    Argument1: ��͂��镶����

  �߂�l(Result)
    -1:    �o�̓t�H�[�}�b�g���ُ킾����
    0�ȏ�: �o�͌��ʂ̍s��

  �߂�l(Value*)
    ��͌���

  �@�\
    �J�^�J�i�œǂ݉�����U��܂��B

  ��
    Argument0: yomi
    Argument1: �@���A�o�����܂��B
    �@��
    Result: 1
    Value0: �L�T���M�A�V���c�Q�L�V�}�X�B


�����ӎ���

dic�ȉ��̎�����ʂ̂��̂ɒu��������ۂ́A����K��UTF-8�ŃR���p�C���������̂��g���Ă��������B
(v1.1��SJIS����ύX���܂���)

NEologd���g�������ꍇ�͈ȉ���URL����R���p�C���ώ������_�E�����[�h�ł��܂��B
http://ssp.shillest.net/etc/mecab-ipadic-neologd-dic.zip
���g��dic�t�H���_�ȉ��ɂ��̂܂܏㏑�����邱�ƂŎg�p�ł��܂��B
�������A�ƂĂ��傫�ȃt�@�C���ɂȂ邽�߁A�g�p����ۂ͏\���������d�˂���łǂ����B

NEologd�ɂ��Ă̏ڍׂȏ��͈ȉ����������������B
https://github.com/neologd/mecab-ipadic-neologd/blob/master/README.ja.md


���z�z������

license.txt�����Ă��������B


���X�V����

�E2016/11/26 ����

�E2017/5/27 v1.1
�@Mecab���C���X�g�[�����ł��܂������Ȃ��ꍇ����������C��
�@�����̕����R�[�h��UTF-8�ɕύX
�@NEologd�œ��쌟��

�E2017/9/10 v1.1.1
�@dicrc�܂ł̃p�X��2�o�C�g�������܂܂��Ƃ��܂��ǂݍ��܂Ȃ������C��

