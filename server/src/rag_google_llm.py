import os
from langchain_google_genai import ChatGoogleGenerativeAI, GoogleGenerativeAIEmbeddings
from langchain_text_splitters import RecursiveCharacterTextSplitter
from langchain_community.vectorstores import Chroma
from langchain_core.prompts import PromptTemplate
from langchain_core.runnables import RunnablePassthrough
from langchain_core.output_parsers import StrOutputParser

# --- 1. SET UP YOUR GOOGLE AI API KEY ---
# Make sure you have a GOOGLE_API_KEY environment variable set.
# You can get a free key from https://aistudio.google.com/
if "GOOGLE_API_KEY" not in os.environ:
    print("Please set the GOOGLE_API_KEY environment variable.")

# --- 2. DEFINE YOUR KNOWLEDGE BASE ---
# This is the data your RAG system will use to answer questions.
# For this example, we'll use a simple string. In a real application,
# you would load this from files (PDFs, TXT, etc.).
knowledge_base_text = """
Gemini is a family of multimodal large language models developed by Google DeepMind.
Announced on December 6, 2023, it is positioned as a competitor to OpenAI's GPT-4.
Gemini is trained to be multimodal, meaning it can understand and process different types of information,
like text, code, images, and video.

The first version, Gemini 1.0, was released in three sizes:
- Ultra: The most powerful and capable model, designed for highly complex tasks. It is the first model to
  outperform human experts on MMLU (Massive Multitask Language Understanding).
- Pro: A high-performing, balanced model, ideal for a wide range of tasks. It is available through
  Google AI Studio and Vertex AI.
- Nano: The most efficient model, designed for on-device tasks. It comes in two sizes, Nano-1 (1.8B parameters)
  and Nano-2 (3.25B parameters).

Gemini Pro is the model powering Google's chatbot, formerly known as Bard.
Access to the Gemini models is available through the Gemini API, Google AI Studio, and Google Vertex AI.
For enterprise-grade use, Vertex AI provides features like security, data privacy, and governance.
"""

# --- 3. INITIALIZE MODELS ---
# We'll use Google's free online models for generation and embeddings.

# LLM for generating answers
# This uses the free Gemini model available via Google AI Studio.
llm = ChatGoogleGenerativeAI(model="gemini-2.5-flash")

# Embedding model
embeddings = GoogleGenerativeAIEmbeddings(model="models/embedding-001")

# --- 4. CREATE THE RAG PIPELINE ---

# a. Split the knowledge base into smaller chunks
text_splitter = RecursiveCharacterTextSplitter(
    chunk_size=1000,
    chunk_overlap=100,
    length_function=len
)
documents = text_splitter.create_documents([knowledge_base_text])
print(f"Split knowledge base into {len(documents)} documents.")

# b. Create a vector store and retriever
# This will embed the document chunks and store them for efficient searching.
# We are using ChromaDB, which is a simple, in-memory vector store.
vector_store = Chroma.from_documents(documents, embeddings)
retriever = vector_store.as_retriever(search_kwargs={"k": 2}) # Retrieve top 2 most relevant chunks
print("Vector store and retriever created.")

# c. Define the prompt template
# This template instructs the LLM how to use the retrieved context to answer the question.
prompt_template = """
You are an expert assistant on Google's Gemini models.
Use the following retrieved context to answer the question.
If you don't know the answer, just say that you don't know.
Keep the answer concise and to the point.

CONTEXT:
{context}

QUESTION:
{question}

ANSWER:
"""
prompt = PromptTemplate(
    template=prompt_template,
    input_variables=["context", "question"]
)

# d. Create the RAG chain using LangChain Expression Language (LCEL)
# This chain defines the flow:
# 1. The user's question is passed to the retriever.
# 2. The retriever finds relevant documents and passes them (along with the original question) to the prompt.
# 3. The formatted prompt is passed to the LLM.
# 4. The LLM generates an answer, which is parsed into a string.
rag_chain = (
    {"context": retriever, "question": RunnablePassthrough()}
    | prompt
    | llm
    | StrOutputParser()
)

print("\nRAG chain created. Ready to answer questions.")

def get_rag_answer(query: str) -> str:
    """
    Invokes the RAG chain with a query and returns the answer.
    This function is designed to be called from other applications (like Rust via PyO3).
    """
    # The RAG chain is already created in the global scope when the module is imported.
    response = rag_chain.invoke(query)
    return response

def main():
    """Main function for standalone command-line execution."""
    import sys
    if len(sys.argv) < 2:
        print("Usage: python rag_google_llm.py \"<your question>\"", file=sys.stderr)
        sys.exit(1)
    query = sys.argv[1]
    print(get_rag_answer(query))

if __name__ == "__main__":
    main()
