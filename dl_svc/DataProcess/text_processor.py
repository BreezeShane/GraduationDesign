"""
    Text processor definition. Convert the text dictionary to Word Embedding.
"""
from os.path import join

PAD_TOKEN = 0   # Used for padding short sentences
SOS_TOKEN = 1   # Start-of-sentence token
EOS_TOKEN = 2   # End-of-sentence token

class Vocabulary:
    """ Vocabulary defining word2index and index2word. """
    def __init__(self, name):
        self.name = name
        self.word2count = {}
        self.word2index = { "PAD": PAD_TOKEN, "SOS": SOS_TOKEN, "EOS": EOS_TOKEN }
        self.index2word = { PAD_TOKEN: "PAD", SOS_TOKEN: "SOS", EOS_TOKEN: "EOS" }
        self.num_words = 3
        self.num_sentences = 0
        self.longest_sentence = 0

    def __call__(self, path):
        """
        Generate a vocabulary from sentence list.
            Args:
                path: The path to the text containing sentences.
        """
        raw_label_lines = None
        with open(path, encoding="utf-8") as f:
            raw_label_lines = f.read().splitlines()
        for sentence in raw_label_lines:
            self.add_sentence(sentence=sentence)

    def get_word2index_dict(self):
        """ Get word2index dict """
        return self.word2index

    def get_index2word_dict(self):
        """ Get index2word dict """
        return self.index2word

    def get_word2count_dict(self):
        """ Get word2count dict """
        return self.word2count

    def add_word(self, word):
        """ Add word to update word2index and word2count. """
        if word not in self.word2index:
            # First entry of word into vocabulary
            self.word2index[word] = self.num_words
            self.word2count[word] = 1
            self.index2word[self.num_words] = word
            self.num_words += 1
        else:
            # Word exists; increase word count
            self.word2count[word] += 1

    def add_sentence(self, sentence):
        """ Extract words from gotten sentence and then update Vocabulary. """
        sentence_len = 0
        for word in sentence.split(' '):
            sentence_len += 1
            self.add_word(word)
        if sentence_len > self.longest_sentence:
            # This is the longest sentence
            self.longest_sentence = sentence_len
        # Count the number of sentences
        self.num_sentences += 1

    def to_word(self, index):
        """ Get word by index. """
        return self.index2word[index]

    def to_index(self, word):
        """ Get index by word. """
        return self.word2index[word]

class Converter:
    """ Convert text file to word vectors. """
    def __init__(self, name, vocabulary: Vocabulary):
        self.name = name
        self.vocabulary = vocabulary
        self.word_vecs = []
        self.max_vec_dim = 0

    def __call__(self, path: str, vec_dim: int):
        """ In the end EOS will be appended in to word vec. """
        raw_lines = None
        with open(path, encoding="utf-8") as f:
            raw_lines = f.read().splitlines()
        for line in raw_lines:
            vector = []
            words = line.split(' ')
            if len(words) > self.max_vec_dim:
                self.max_vec_dim = len(words)
            for word in words:
                vector.append(self.vocabulary.to_index(word))
            vector = self._fill_vec(vector, vec_dim)
            self.word_vecs.append(vector)
        self._fill_eos()

    def _fill_eos(self):
        eos_value = self.vocabulary.to_index("EOS")
        for word_vec in self.word_vecs:
            word_vec.append(eos_value)
        self.max_vec_dim += 1

    def _fill_vec(self, vec2cvrt, exp_vec_dim: int):
        curr_vec_dim = len(vec2cvrt)
        if curr_vec_dim > exp_vec_dim:
            raise ValueError("The dimention of word vector is too little to contain it.\n"
                             f"Max word vector dimention is: {self.max_vec_dim}")
        elif curr_vec_dim < exp_vec_dim:
            count_to_fill = exp_vec_dim - curr_vec_dim
            pad_value = self.vocabulary.to_index("PAD")
            for _ in range(count_to_fill):
                vec2cvrt.append(pad_value)
        return vec2cvrt

    def get_max_dim_in_word_vecs(self):
        """ Get the max dimention in word vectors """
        return self.max_vec_dim

    def get_word_vecs(self):
        """ Get word vectors in list. """
        return self.word_vecs


def text_process(arguments, record_file):
    """ Process text file to Vocabulary Dict and WordVec Dict. """
    species_file_path = join(arguments.text, record_file)
    vocab = Vocabulary("SpeciesDict")
    cvrt = Converter("SpeciesVectorsDict", vocab)
    vocab(species_file_path)
    cvrt(species_file_path, vec_dim=4)
    return vocab, cvrt
